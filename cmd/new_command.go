package cmd

import (
	log "github.com/sirupsen/logrus"
	"github.com/spf13/cobra"

	"github.com/planetA/ovey/lib"
)

var newCmd = &cobra.Command{
	Use:   OveyNewUse,
	Short: OveyNewShort,
	Long:  OveyNewLong,

	Run: func(cmd *cobra.Command, args []string) {
		err := lib.NewDevice(virtDeviceName, parentDeviceName)
		if err != nil {
			log.WithError(err).Fatal("Failed to create new device")
		}
	},
}

func init() {

	newCmd.Flags().StringVarP(&virtDeviceName, "name", "n", "", "Name of the new virtual device")
	newCmd.MarkFlagRequired("name")
	newCmd.Flags().StringVarP(&parentDeviceName, "parent", "p", "", "Name of the parent device")
	newCmd.MarkFlagRequired("parent")

	rootCmd.AddCommand(newCmd)
}
