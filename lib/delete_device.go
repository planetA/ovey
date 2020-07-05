package lib

import (
	"fmt"

	"github.com/mdlayher/netlink"
)

func DeleteDevice(virtDeviceName string) error {
	ol, err := Dial()
	if err != nil {
		return fmt.Errorf("failed to dial generic netlink: %v", err)
	}
	defer ol.Close()

	ae := netlink.NewAttributeEncoder()
	ae.String(oveyAttributeVirtDevice, virtDeviceName)
	data, err := ae.Encode()
	if err != nil {
		return fmt.Errorf("Encoding failed: %v", err)
	}

	req := ol.NewRequest(oveyCommandDeleteDevice, data)
	_, err = req.Execute()
	if err != nil {
		return fmt.Errorf("failed to send a message: %v", err)
	}

	return nil
}
