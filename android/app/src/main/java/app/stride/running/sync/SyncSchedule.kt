package app.stride.running.sync

import android.content.Context
import java.time.Instant
import java.time.ZoneId
import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle

/**
 * The phone's next pull time.
 *
 * WorkManager runs periodic work inexactly — the system decides when the
 * constraints allow it — so the phone cannot promise a second on the
 * clock. It can name the next local boundary it has scheduled around. The
 * default six-hour cadence is anchored at 02:00, which gives 02:00, 08:00,
 * 14:00, and 20:00 in the phone's own time zone.
 *
 * The calculation is separate from Android, so the rule can be tested
 * without a phone.
 */
object SyncSchedule {

    /** The first pull boundary of a local day. */
    const val ANCHOR_HOUR = 2

    /**
     * The next pull boundary after an instant.
     *
     * A boundary that is exactly now has passed: the next one is in the
     * following interval.
     */
    fun nextBoundary(
        now: ZonedDateTime,
        intervalHours: Long,
    ): ZonedDateTime {
        val interval = intervalHours.coerceIn(1, 24)
        val midnight = now.toLocalDate().atStartOfDay(now.zone)
        var candidate = midnight.plusHours(ANCHOR_HOUR.toLong())

        while (!candidate.isAfter(now)) {
            candidate = candidate.plusHours(interval)
        }
        return candidate
    }

    /** The next pull boundary after an epoch millisecond. */
    fun nextBoundary(
        nowMillis: Long,
        intervalHours: Long,
        zone: ZoneId = ZoneId.systemDefault(),
    ): Long = nextBoundary(
        Instant.ofEpochMilli(nowMillis).atZone(zone),
        intervalHours,
    ).toInstant().toEpochMilli()

    /** The local time of a scheduled pull, in the phone's locale. */
    fun formatTime(context: Context, atMillis: Long): String {
        val formatter = DateTimeFormatter
            .ofLocalizedTime(FormatStyle.SHORT)
            .withLocale(context.resources.configuration.locales[0])
        return Instant.ofEpochMilli(atMillis)
            .atZone(ZoneId.systemDefault())
            .format(formatter)
    }
}
