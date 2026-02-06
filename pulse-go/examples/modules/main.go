package main

import (
	"fmt"

	"github.com/machanirobotics/pulse/pulse-go"
)

func main() {
	// ========================================
	// 1. Main service — uses global logging.level from pulse.toml (Level 2 = Info)
	// ========================================
	core, err := pulse.New().
		WithService("robot-core", "1.0.0").
		Build()
	if err != nil {
		panic(err)
	}
	defer core.Close()

	core.Logger.Info("=== Robot Core Started ===")
	core.Logger.Info("Global level is 2 (Info) from pulse.toml")
	core.Logger.Debug("This debug message is HIDDEN because global level=2 (Info)")
	core.Logger.Info("Core system initialized")
	core.Logger.Error("Core error test — always visible")

	// ========================================
	// 2. Module: nats-module — TOML overrides to Level 1 (Error only)
	//    Code says Level3, but pulse.toml has [logging.modules.nats-module] level=1
	//    TOML wins → only errors show
	// ========================================
	fmt.Println("\n--- nats-module (TOML override → Level 1 = Error only) ---")

	nats, err := pulse.New().
		WithService("nats-module", "0.2.0").
		WithLogLevel(pulse.ModuleLevel_3). // Code says Level3 (Debug)
		Build()
	if err != nil {
		panic(err)
	}
	defer nats.Close()

	nats.Logger.Debug("NATS debug — HIDDEN (TOML overrode to Level 1)")
	nats.Logger.Info("NATS info — HIDDEN (TOML overrode to Level 1)")
	nats.Logger.Warn("NATS warn — HIDDEN (TOML overrode to Level 1)")
	nats.Logger.Error("NATS error — VISIBLE (Level 1 = Error only)")

	// ========================================
	// 3. Module: vision-module — no TOML override, code sets Level 3 (Debug)
	//    WithLogLevel(Level3) wins → full debug output
	// ========================================
	fmt.Println("\n--- vision-module (code default → Level 3 = Debug) ---")

	vision, err := pulse.New().
		WithService("vision-module", "0.1.0").
		WithLogLevel(pulse.ModuleLevel_3). // Code says Level3, no TOML override → Level3 wins
		Build()
	if err != nil {
		panic(err)
	}
	defer vision.Close()

	vision.Logger.Debug("Vision debug — VISIBLE (Level 3)")
	vision.Logger.Info("Vision info — VISIBLE (Level 3)")
	vision.Logger.Warn("Vision warn — VISIBLE (Level 3)")
	vision.Logger.Error("Vision error — VISIBLE (Level 3)")

	// ========================================
	// 4. Module: motor-module — no TOML override, no WithLogLevel
	//    Falls back to global logging.level=2 from pulse.toml → Info
	// ========================================
	fmt.Println("\n--- motor-module (no override → global Level 2 = Info) ---")

	motor, err := pulse.New().
		WithService("motor-module", "1.5.0").
		Build()
	if err != nil {
		panic(err)
	}
	defer motor.Close()

	motor.Logger.Debug("Motor debug — HIDDEN (global Level 2 = Info)")
	motor.Logger.Info("Motor info — VISIBLE (Level 2)")
	motor.Logger.Warn("Motor warn — VISIBLE (Level 2)")
	motor.Logger.Error("Motor error — VISIBLE (Level 2)")

	// ========================================
	// Summary
	// ========================================
	fmt.Println("\n=== Priority Chain Summary ===")
	fmt.Println("  env var (PULSE_LOGGING_MODULES_<NAME>_LEVEL)")
	fmt.Println("    > TOML [logging.modules.<name>] level")
	fmt.Println("      > WithLogLevel() in code")
	fmt.Println("        > global [logging] level")
	fmt.Println("          > environment-based default")
}
