package app.stride.running.setup

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * The one link on the setup screen.
 *
 * The link goes to Garmin's page for a Personal API token. It is the only
 * place the key can be made, and a key page on another host would be a page
 * that collects the key — so the host is checked, here, where a test can
 * hold it.
 */
class GarminKeyPageTest {

    @Test
    fun `the link is Garmin's key page`() {
        assertTrue(GarminKeyPage.isGarminKeyPage(GarminKeyPage.URL))
        assertTrue(GarminKeyPage.URL.startsWith("https://connect.garmin.com/"))
    }

    @Test
    fun `another host is not the key page`() {
        assertFalse(GarminKeyPage.isGarminKeyPage("https://example.com/"))
        assertFalse(GarminKeyPage.isGarminKeyPage("http://connect.garmin.com/app/settings"))
        assertFalse(GarminKeyPage.isGarminKeyPage("https://connect.garmin.com.evil.test/"))
    }
}
