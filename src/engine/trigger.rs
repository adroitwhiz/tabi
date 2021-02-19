#[derive(Debug, PartialEq)]
pub enum Trigger {
    WhenFlagClicked,
    WhenSpriteClicked,
    WhenKeyPressed(char),
    WhenBackdropSwitches(String),
    WhenIReceive(String),
    WhenIStartAsAClone,
}
