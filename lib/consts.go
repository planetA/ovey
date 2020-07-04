package lib

const oveyFamilyName = "rdma-ovey"

const (
	oveyCommandEcho = iota + 1
	oveyCommandNewDevice
)

const (
	oveyAttributeMsg = iota + 1
	oveyAttributeNewDevice
	oveyAttributeParentDevice
)
