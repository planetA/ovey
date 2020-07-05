package cmd

import (
	log "github.com/sirupsen/logrus"
	"github.com/spf13/cobra"

	"github.com/planetA/ovey/lib"
)

var deleteCmd = &cobra.Command{
	Use:   OveyDeleteUse,
	Short: OveyDeleteShort,
	Long:  OveyDeleteLong,

	Run: func(cmd *cobra.Command, args []string) {
		err := lib.DeleteDevice(virtDeviceName)
		if err != nil {
			log.WithError(err).Fatal("Failed to create delete device")
		}
	},
}

func init() {

	deleteCmd.Flags().StringVarP(&virtDeviceName, "name", "n", "", "Name of the delete virtual device")
	deleteCmd.MarkFlagRequired("name")

	rootCmd.AddCommand(deleteCmd)
}
