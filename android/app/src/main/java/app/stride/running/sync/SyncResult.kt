package app.stride.running.sync

import org.json.JSONObject

/**
 * What a pull brought in, read out of the api's DTO.
 *
 * **The fields that exist, and only those.**
 *
 * The api's `SyncRun` is:
 *
 * ```text
 * { activities, wellness_days, finished_at, generated_at }
 * ```
 *
 * and its `SyncStatus` is:
 *
 * ```text
 * { last_sync_at, last_error, activity_count }
 * ```
 *
 * A field that is not in the DTO reads as zero, and a phone that says "0
 * runs" after a real pull is worse than no phone at all — so the reads are
 * here, in one place, with the DTO's names written on them, and the test
 * holds them to the DTO.
 *
 * It has no Android in it, so it can be tested without a phone.
 */
object SyncResult {

    /**
     * What a pull brought in.
     *
     * @property activities how many runs the pull brought in.
     * @property wellnessDays how many days of wellness the pull brought in.
     */
    data class Counts(
        val activities: Int,
        val wellnessDays: Int,
    )

    /**
     * The counts of a pull, out of the api's `SyncRun`.
     */
    fun counts(body: JSONObject?): Counts {
        if (body == null) return Counts(0, 0)
        return Counts(
            activities = body.optInt("activities", 0),
            wellnessDays = body.optInt("wellness_days", 0),
        )
    }

    /**
     * How many runs the phone has, out of the api's `SyncStatus`.
     *
     * It is `activity_count` — the number of runs the store holds — and
     * not the length of a page of the feed: a page of ten is not the
     * phone's history.
     */
    fun storedRuns(body: JSONObject?): Int {
        if (body == null) return 0
        return body.optInt("activity_count", 0)
    }
}
