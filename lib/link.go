package lib

import (
	"errors"
	"fmt"
	"os"

	"github.com/mdlayher/genetlink"
	"github.com/mdlayher/netlink"
)

type oveyRequest struct {
	message genetlink.Message
	flags   netlink.HeaderFlags
	link      *oveyLink
}

func (or *oveyRequest) Execute() ([]genetlink.Message, error) {
	return or.link.conn.Execute(or.message, or.link.family.ID, or.flags)
}

type oveyLink struct {
	conn   *genetlink.Conn
	family genetlink.Family
}

func Dial() (*oveyLink, error) {
	c, err := genetlink.Dial(nil)
	if err != nil {
		return nil, fmt.Errorf("Failed to dial genetlink", err)
	}
	defer func() {
		if err != nil {
			c.Close()
		}
	}()

	// Ask generic netlink about the generic netlink controller (nlctrl)'s
	// family information.
	family, err := c.GetFamily(oveyFamilyName)
	if err != nil {
		// If a family doesn't exist, the error can be checked using
		// errors.Is in Go 1.13+, or the deprecated netlink.IsNotExist in Go
		// 1.12 and below.
		if errors.Is(err, os.ErrNotExist) {
			return nil, fmt.Errorf("%q family not available", oveyFamilyName)
		}

		return nil, fmt.Errorf("failed to query family %q: %v", oveyFamilyName, err)
	}

	return &oveyLink{
		conn:   c,
		family: family,
	}, nil
}

func (ol *oveyLink) NewRequest(command uint8, data []byte) *oveyRequest {
	return &oveyRequest{
		message: genetlink.Message{
			Header: genetlink.Header{
				Command: command,
				Version: ol.family.Version,
			},
			Data: data,
		},
		flags: netlink.Request,
		link:  ol,
	}

}

func (ol *oveyLink) Close() {
	ol.conn.Close()
}
