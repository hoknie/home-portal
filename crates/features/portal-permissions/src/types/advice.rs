use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Advice {
    AllowInSettings,
    AnswerThePrompt,
    ConnectAVolume,
    GrantFullDiskAccess,
    ApplicationNotFound,
    CheckFailed,
}

impl Advice {
    pub fn code(self) -> &'static str {
        match self {
            Advice::AllowInSettings => "allow-in-settings",
            Advice::AnswerThePrompt => "answer-the-prompt",
            Advice::ConnectAVolume => "connect-a-volume",
            Advice::GrantFullDiskAccess => "grant-full-disk-access",
            Advice::ApplicationNotFound => "application-not-found",
            Advice::CheckFailed => "check-failed",
        }
    }

    pub fn text(self) -> &'static str {
        match self {
            Advice::AllowInSettings => "allow it in System Settings",
            Advice::AnswerThePrompt => {
                "answer the system prompt on the Mac, or allow it in System Settings"
            }
            Advice::ConnectAVolume => "connect a removable volume and ask again",
            Advice::GrantFullDiskAccess => "add the process in System Settings",
            Advice::ApplicationNotFound => {
                "the application was not found or could not be started; check its name"
            }
            Advice::CheckFailed => "the check did not finish; ask again",
        }
    }
}
