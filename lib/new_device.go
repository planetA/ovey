package lib

import (
	"fmt"

	"github.com/mdlayher/netlink"
	log "github.com/sirupsen/logrus"
)

func NewDevice(newDeviceName, parentDeviceName string) error {
	ol, err := Dial()
	if err != nil {
		return fmt.Errorf("failed to dial generic netlink: %v", err)
	}
	defer ol.Close()

	ae := netlink.NewAttributeEncoder()
	ae.String(oveyAttributeNewDevice, newDeviceName)
	ae.String(oveyAttributeParentDevice, parentDeviceName)
	data, err := ae.Encode()
	if err != nil {
		return fmt.Errorf("Encoding failed: %v", err)
	}

	req := ol.NewRequest(oveyCommandNewDevice, data)
	resp, err := req.Execute()
	if err != nil {
		return fmt.Errorf("failed to send a message: %v", err)
	}

	for _, m := range resp {
		ad, err := netlink.NewAttributeDecoder(m.Data)
		if err != nil {
			return fmt.Errorf("failed to create attribute decoder: %v", err)
		}

		for ad.Next() {
			switch ad.Type() {
			case oveyAttributeMsg:
				log.Info("Received message: ", ad.String())
			}
		}
		log.Printf("Received message: %+v", m)
	}

	return nil
}
