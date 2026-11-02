package app.stride.running.sync

import android.content.Context
import app.stride.running.core.StrideCore
import app.stride.running.security.Secrets

/**
 * Forgetting the Garmin key.
 *
 * **Three things, and nothing else.**
 *
 * 1. the scheduled and the queued work stop — there is nothing to pull with;
 * 2. the Keystore-wrapped envelope goes, and the wrapping key with it;
 * 3. the core's in-memory copy of the key goes, so the next call cannot
 *    use it.
 *
 * **The runs stay.** Forgetting the key is not forgetting the data: the
 * activities, wellness and plans live in the phone's own database, and
 * nothing here deletes one. The database has its own Keystore-wrapped key,
 * which this operation never touches.
 *
 * The order is the order of the blast radius: the work stops first, so no
 * new pull can start while the key is being taken away; the envelope goes
 * next; the core's copy goes last, because the core is what a pull reads
 * from.
 *
 * Nothing here returns, logs or transmits the key. These steps are about
 * the key's *existence*, and the key itself is never in this file's hands:
 * the envelope is cleared, not read.
 */
object ForgetGarminKey {

    /**
     * What the operation removed, and what it deliberately kept.
     *
     * The report is made from the steps that actually ran. It never
     * contains the key or any part of it other than the identifier the
     * native settings surface may already have shown.
     */
    data class Report(
        /** The scheduled and queued Garmin work was cancelled. */
        val scheduledPullsStopped: Boolean,

        /** The encrypted Garmin credential envelope is gone. */
        val credentialEnvelopeRemoved: Boolean,

        /** The non-exportable Keystore alias is gone. */
        val wrappingKeyRemoved: Boolean,

        /** The core's temporary in-memory copy is gone. */
        val memoryCopyCleared: Boolean,

        /** The local database, runs, wellness and plans are still here. */
        val databaseKept: Boolean = true,
    )

    /**
     * Forget the key on the phone.
     *
     * The Android that carries the three steps out. Every step is
     * attempted, even if an earlier one fails, so one broken part cannot
     * leave the credential in another part of the process.
     */
    fun onPhone(context: Context): Report {
        var scheduledPullsStopped = false
        var envelopeRemoved = false
        var wrappingKeyRemoved = false
        var memoryCopyCleared = false

        run(
            stopScheduledWork = {
                SyncScheduler.cancel(context)
                scheduledPullsStopped = true
            },
            clearEnvelope = {
                val removal = Secrets.forgetGarminKey(context)
                envelopeRemoved = removal.envelopeRemoved
                wrappingKeyRemoved = removal.wrappingKeyRemoved
            },
            clearCoreKey = {
                // Two things: the copy in the core's memory goes, and the
                // phone says it is not connected any more. Clearing the copy
                // is not the same as disconnecting — that is what the key in
                // the Keystore is for — so the fact is said separately.
                StrideCore.setGarminKey("")
                StrideCore.setGarminConnected(false)
                memoryCopyCleared = true
            },
        )

        return Report(
            scheduledPullsStopped = scheduledPullsStopped,
            credentialEnvelopeRemoved = envelopeRemoved,
            wrappingKeyRemoved = wrappingKeyRemoved,
            memoryCopyCleared = memoryCopyCleared,
        )
    }

    /**
     * The three steps, with the actions given.
     *
     * The order is what the test pins down. **All three are attempted**, even
     * if one of them fails: a Keystore that refuses the envelope must not
     * leave the key in the core's memory, and a work queue that will not
     * cancel must not leave the envelope in place. The first failure is
     * the one reported.
     */
    fun run(
        stopScheduledWork: () -> Unit,
        clearEnvelope: () -> Unit,
        clearCoreKey: () -> Unit,
    ) {
        var failure: Throwable? = null
        for (step in listOf(stopScheduledWork, clearEnvelope, clearCoreKey)) {
            try {
                step()
            } catch (error: Throwable) {
                if (failure == null) failure = error
            }
        }
        failure?.let { throw it }
    }
}
