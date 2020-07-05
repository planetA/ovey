package lib

const oveyFamilyName = "rdma-ovey"

const (
	oveyCommandEcho = iota + 1
	oveyCommandNewDevice
	oveyCommandDeleteDevice
)

const (
	oveyAttributeMsg = iota + 1
	oveyAttributeVirtDevice
	oveyAttributeParentDevice
)
