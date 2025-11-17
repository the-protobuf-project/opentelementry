package options

// ProfilingOptions defines the settings for continuous profiling with Pyroscope
type ProfilingOptions struct {
	Enabled       bool              `json:"enabled"`       // Enable continuous profiling
	ServerAddress string            `json:"serverAddress"` // Pyroscope server URL (e.g., "http://localhost:4040")
	
	// Authentication (optional, required for Grafana Cloud)
	BasicAuthUser     string `json:"basicAuthUser"`     // Basic auth username
	BasicAuthPassword string `json:"basicAuthPassword"` // Basic auth password
	TenantID          string `json:"tenantId"`          // Tenant ID for multi-tenancy (optional)
	
	// Profile types - enable/disable specific profiling types
	ProfileCPU            bool `json:"profileCpu"`            // CPU profiling (default: true)
	ProfileAllocObjects   bool `json:"profileAllocObjects"`   // Allocation objects profiling (default: true)
	ProfileAllocSpace     bool `json:"profileAllocSpace"`     // Allocation space profiling (default: true)
	ProfileInuseObjects   bool `json:"profileInuseObjects"`   // In-use objects profiling (default: true)
	ProfileInuseSpace     bool `json:"profileInuseSpace"`     // In-use space profiling (default: true)
	ProfileGoroutines     bool `json:"profileGoroutines"`     // Goroutines profiling (default: false)
	ProfileMutexCount     bool `json:"profileMutexCount"`     // Mutex count profiling (default: false)
	ProfileMutexDuration  bool `json:"profileMutexDuration"`  // Mutex duration profiling (default: false)
	ProfileBlockCount     bool `json:"profileBlockCount"`     // Block count profiling (default: false)
	ProfileBlockDuration  bool `json:"profileBlockDuration"`  // Block duration profiling (default: false)
	
	// Profile rates
	MutexProfileRate int `json:"mutexProfileRate"` // Mutex profile fraction (e.g., 5 = 1/5 events reported)
	BlockProfileRate int `json:"blockProfileRate"` // Block profile rate in nanoseconds (e.g., 5)
	
	// Custom tags (optional)
	Tags map[string]string `json:"tags"` // Additional tags to attach to profiles
}
