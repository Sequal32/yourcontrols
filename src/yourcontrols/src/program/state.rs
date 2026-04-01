use super::emulator_runtime::EmulatorRuntimeState;

#[derive(Default)]
pub struct ProgramState {
    pub(crate) auto_start_pending: bool,
    pub emulator: EmulatorRuntimeState,
}
