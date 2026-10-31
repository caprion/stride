package app.stride.running.security

/**
 * Whether the app is open.
 *
 * The app gate is a *foreground* gate: it stops a borrowed phone from showing
 * the athlete's history. It is not a key of its own — the database key is in
 * the Keystore, and the Keystore already refuses to work while the phone is
 * locked.
 *
 * **The bridge asks this.** A locked screen gets no real data: the bridge
 * refuses the routes that carry the athlete's history while the gate is
 * closed, so the rule is enforced on the way in rather than by an overlay
 * that a clever page could paint over.
 */
object AppLock {

    @Volatile
    private var unlocked = false

    /** The gate is open. */
    fun setUnlocked(value: Boolean) {
        unlocked = value
    }

    /** Whether the gate is open. */
    fun isUnlocked(): Boolean = unlocked
}
