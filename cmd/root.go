package cmd

import (
	"os"

	log "github.com/sirupsen/logrus"
	"github.com/spf13/cobra"
)

var (
	logLevel int

	rootCmd = &cobra.Command{
		TraverseChildren: true,

		Use:   OveyUse,
		Short: OveyShort,
		Long:  OveyLong,

		PersistentPreRun: func(cmd *cobra.Command, args []string) {
			if logLevel > int(log.TraceLevel) {
				logLevel = int(log.TraceLevel)
			}
			log.SetLevel(log.Level(logLevel))
		},
	}
)

func Execute() {
	if err := rootCmd.Execute(); err != nil {
		log.WithError(err).Fatal("Failed")
		os.Exit(1)
	}
}

func init() {
	rootCmd.PersistentFlags().CountVarP(&logLevel, "verbose", "v", "Increase verbosity level")
}
