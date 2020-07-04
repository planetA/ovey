package lib

import (
	"errors"
	"fmt"
	"os"

	"github.com/mdlayher/genetlink"
	"github.com/mdlayher/netlink"
	log "github.com/sirupsen/logrus"
)

func Echo() {
	fmt.Println("hello world")
	c, err := genetlink.Dial(nil)
	if err != nil {
		log.Fatalf("failed to dial generic netlink: %v", err)
	}
	defer c.Close()

	// Ask generic netlink about all families registered with it.
	families, err := c.ListFamilies()
	if err != nil {
		log.Fatalf("failed to query for families: %v", err)
	}

	for i, f := range families {
		log.Printf("#%02d: %+v", i, f)
	}

	// Ask generic netlink about the generic netlink controller (nlctrl)'s
	// family information.
	const name = "rdma-ovey"
	family, err := c.GetFamily(name)
	if err != nil {
		// If a family doesn't exist, the error can be checked using
		// errors.Is in Go 1.13+, or the deprecated netlink.IsNotExist in Go
		// 1.12 and below.
		if errors.Is(err, os.ErrNotExist) {
			log.Printf("%q family not available", name)
			return
		}

		log.Fatalf("failed to query for family: %v", err)
	}
	log.Printf("%s: %+v", name, family)

	req := genetlink.Message{
		Header: genetlink.Header{
			Command: oveyCommandEcho,
			Version: family.Version,
		},
	}
	flags := netlink.Request

	resp, err := c.Execute(req, family.ID, flags)
	if err != nil {
		log.Fatalf("failed to send a message: %v", err)
	}

	for _, m := range resp {
		ad, err := netlink.NewAttributeDecoder(m.Data)
		if err != nil {
			log.Fatalf("failed to create attribute decoder: %v", err)
		}

		for ad.Next() {
			switch ad.Type() {
			case oveyAttributeMsg:
				log.Info("Received message: ", ad.String())
			}
		}
		log.Printf("Received message: %+v", m)
	}
}
