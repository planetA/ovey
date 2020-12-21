//! Crate-private handlers for the REST API of Ovey Daemon. They get invoked by Ovey CLI.

use actix_web::{web, HttpRequest, HttpResponse};
use liboveyutil::guid::{guid_string_to_u64, guid_u64_to_string};
use ovey_daemon::errors::DaemonRestError;
use ovey_daemon::structs::{
    CreateDeviceInput, CreateDeviceInputBuilder, DeleteDeviceInput, DeleteDeviceInputBuilder,
    DeletionStateDto, DeletionStateDtoBuilder
};
use crate::coordinator_service::{
    rest_check_device_is_allowed, rest_forward_create_device, rest_forward_delete_device
};
use crate::OCP;
use liboveyutil::types::Uuid;

pub async fn route_get_index(_req: HttpRequest) -> HttpResponse {
    //println!("request: {:?}", &req);
    HttpResponse::Ok().json("STATUS: UP") // <- send response
}

pub async fn route_post_create_device(
    input: web::Json<CreateDeviceInput>,
) -> Result<actix_web::HttpResponse, DaemonRestError> {
    // verify input
    let input: Result<CreateDeviceInput, String> = CreateDeviceInputBuilder::rebuild(input.0);
    let input = input.map_err(|e| DaemonRestError::MalformedPayload(e))?;

    // FIRST STEP: check if device is allowed by coordinator
    let is_allowed = rest_check_device_is_allowed(input.network_id(), input.virt_guid()).await?;

    if !is_allowed {
        return Err(DaemonRestError::DeviceNotAllowed {
            network_id: input.network_id().to_owned(),
            virt_guid: input.virt_guid().to_owned(),
        });
    }

    // SECOND STEP: REGISTER DEVICE LOCALLY VIA OCP INSIDE KERNEL
    // now we first create the device on the machine
    // and then we tell the coordinator about it

    // TODO VERY IMPORTANT TODO WHEN THE THREAD GETS STUCK HERE, THE LISTENING THREAD CAN'T GET THE LOCK TO
    //  GET INCOMING REQUESTS!
    let mut ocp = OCP.lock().unwrap();
    // check if device exists already in kernel
    let ocp_res = ocp.ocp_get_device_info(input.device_name());
    if ocp_res.is_ok() && ocp_res.unwrap().device_name().is_some() {
        return Err(DaemonRestError::OcpDeviceAlreadyExists {
            info: format!(
                "The device {} already exists inside the kernel!",
                input.device_name()
            ),
        });
    }

    let guid_be = guid_string_to_u64(input.virt_guid());
    let ocp_res = ocp.ocp_create_device(
        input.device_name(),
        input.parent_device_name(),
        guid_be,
        &input.network_id().to_string(),
    );
    // now fetch again data; we need the parent device guid
    let _ocp_res = ocp_res.map_err(|err| DaemonRestError::OcpOperationFailed {
        info: format!("OCP could not create device! {}", err),
    })?;
    let ocp_res = ocp
        .ocp_get_device_info(input.device_name())
        .map_err(|err| DaemonRestError::OcpOperationFailed {
            info: format!("OCP could not get device data! {}", err),
        })?;

    // THIRD STEP: NOW REGISTER THE DEVICE AT COORDINATOR
    let device_name = input.device_name().to_owned(); // fix use after move with input.device_name() later needed
    let resp = rest_forward_create_device(
        input,
        guid_u64_to_string(
            ocp_res
                .parent_node_guid()
                .expect("Must exist at this point"),
        ),
    )
    .await;

    // if something failed; delete device on local machine again
    if resp.is_err() {
        eprintln!("A failure occurred: {:#?}", resp.as_ref().unwrap_err());
        let ocp_res = ocp.ocp_delete_device(&device_name);
        if let Err(err) = ocp_res {
            return Err(DaemonRestError::OcpOperationFailed {
                info: format!("Local device deletion failed! {}", err),
            });
        }
    }

    let dto = resp?;

    Ok(HttpResponse::Ok().json(dto))
}

pub async fn route_delete_delete_device(
    input: web::Json<DeleteDeviceInput>,
) -> Result<actix_web::HttpResponse, DaemonRestError> {
    // verify input
    let input: Result<DeleteDeviceInput, String> = DeleteDeviceInputBuilder::rebuild(input.0);
    let input = input.map_err(|e| DaemonRestError::MalformedPayload(e))?;

    // first step; check via OCP if device is registered on local machine
    let mut ocp = OCP.lock().unwrap();
    let ocp_data = ocp.ocp_get_device_info(input.device_name()).map_err(|_err| {
        DaemonRestError::OcpDeviceNotFound {
            info: input.device_name().to_owned(),
        }
    });
    let ocp_data = ocp_data?;

    // second step: delete it on coordinator
    // fetch network id; we need it for the deletion request on the coordinator
    let network_id = ocp_data.virt_network_uuid_str().unwrap();
    let network_id =
        Uuid::parse_str(network_id).map_err(|err| DaemonRestError::OtherInternalError {
            info: err.to_string(),
        })?;
    let guid_str = guid_u64_to_string(ocp_data.node_guid().unwrap());

    // delete in both places without early canceling (no .unwrap() or ?)

    let coordinator_result = rest_forward_delete_device(&guid_str, &network_id).await;

    // actually delete device on local machine inside Ovey kernel module
    let ocp_result = ocp.ocp_delete_device(input.device_name()).map_err(|err| {
        DaemonRestError::OcpOperationFailed {
            info: format!("Local device deletion failed! {}", err),
        }
    });

    let deletion_state: DeletionStateDto = DeletionStateDtoBuilder::default()
        .deletion_local_successfully(ocp_result.is_ok())
        .deletion_local_info_msg(
            ocp_result
                .err()
                // display is implemented by a derive macro
                // even if IDE doesn't recognize it
                .map(|e| format!("{}", e)),
        )
        .deletion_coordinator_successfully(coordinator_result.is_ok())
        .deletion_coordinator_info_msg(
            coordinator_result
                .err()
                // display is implemented by a derive macro
                // even if IDE doesn't recognize it
                .map(|e| format!("{}", e)),
        )
        .build()
        .unwrap();

    Ok(
        HttpResponse::Ok().json(deletion_state)
    )
}
