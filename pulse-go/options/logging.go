package options

type TimeFormat string

const (
	TimeFormatRFC3339     TimeFormat = "RFC3339"
	TimeFormatRFC3339Nano TimeFormat = "RFC3339Nano"
	TimeFormatKitchen     TimeFormat = "Kitchen"
	TimeFormatStamp       TimeFormat = "Stamp"
	TimeFormatCustom      TimeFormat = "Custom"
)

type LogOptions struct {
	Prefix          string     // Prefix string in log output
	ReportCaller    bool       // Include caller info in logs
	ReportTimestamp bool       // Include timestamp in logs
	TimeFormatKey   TimeFormat // Named time format enum
	CustomFormat    string     // Custom format if TimeFormatKey == TimeFormatCustom
	CallerOffset    int        // Adjust call depth for correct file/line display
}

type LoggingOptions struct {
	Log LogOptions `json:"log"` // Log options
}
