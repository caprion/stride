package app.stride.running.setup

/**
 * Where the Garmin key comes from.
 *
 * **One page, one URL.** Garmin's own page for a Personal API token is the
 * only place the key can be made, and it is the only place the screen
 * sends the athlete to.
 *
 * It is here, and not in the activity, because it is a fact about Garmin
 * and not about Android: the activity is the only thing that knows how to
 * open a browser, and the test that guards the URL is a plain JVM test.
 */
object GarminKeyPage {

    /** Garmin's page for a Personal API token. */
    const val URL = "https://connect.garmin.com/app/settings/personalApiToken"

    /**
     * Whether a URL is Garmin's key page.
     *
     * The screen only ever opens this one, and the check is here so that a
     * test can hold it: a key page on another host would be a page that
     * collects the key.
     */
    fun isGarminKeyPage(url: String): Boolean = url.startsWith("https://connect.garmin.com/")
}
