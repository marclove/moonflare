// Integration tests are now split across multiple modules for better organization:
// - workspace_tests.rs: Basic workspace creation and structure tests
// - typescript_tests.rs: TypeScript project-specific tests
// - wasm_integration_tests.rs: WASM build and distribution tests
// - property_tests.rs: Property-based testing with random project sequences
//
// All shared utilities, types, and verification functions are in the common module.
