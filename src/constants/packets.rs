/// Login replys (i32)
///
/// ```
/// LoginReply {
///     InvalidCredentials        = -1,
///     OutdatedClient            = -2,
///     UserBanned                = -3,
///     MultiaccountDetected      = -4,
///     ServerError               = -5,
///     CuttingEdgeMultiplayer    = -6,
///     AccountPasswordRest       = -7,
///     VerificationRequired      = -8
/// }
///
/// ```
///
pub enum LoginReply {
    InvalidCredentials = -1,
    OutdatedClient = -2,
    UserBanned = -3,
    MultiaccountDetected = -4,
    ServerError = -5,
    CuttingEdgeMultiplayer = -6,
    AccountPasswordRest = -7,
    VerificationRequired = -8,
}
