pub enum RunnerEvent {
    Line(String),
    Exited,
    Complete,
    SpawnError(String),
    Resize(u16, u16),
}
