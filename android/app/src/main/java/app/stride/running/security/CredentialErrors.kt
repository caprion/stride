package app.stride.running.security

/**
 * The two ways a credential can fail to be read.
 *
 * They are different problems and the app answers them differently: one is
 * "come back later", the other is "this is gone".
 */

/**
 * The wrapping key cannot be used right now.
 *
 * The usual reason is that the phone is locked: a key that requires an
 * unlocked device is unusable while it is locked. The credential is fine and
 * will be readable again the moment the phone is unlocked.
 */
class CredentialUnavailable(message: String, cause: Throwable? = null) :
    Exception(message, cause)

/**
 * The wrapping key is gone.
 *
 * The keystore invalidated it — the device was wiped, or the key was
 * invalidated with the lock state — so the file it protected can never be
 * read again. The app says so and asks the athlete to connect again; it
 * does not pretend the file is empty, because it is not.
 */
class CredentialUnreadable(message: String, cause: Throwable? = null) :
    Exception(message, cause)
