package app.stride.running.bridge

import app.stride.running.bridge.PhoneRoutes.Decision
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * What the phone does with each route.
 *
 * The routing lives in Rust; these are the phone's own decisions, and they
 * are the ones that keep the credential out of the WebView.
 */
class PhoneRoutesTest {

    @Test
    fun `the Garmin key never travels through the page`() {
        assertEquals(
            Decision.GarminKeyEntry,
            PhoneRoutes.decide("POST", "/api/v1/auth/garmin/personal-key"),
        )
        assertEquals(
            Decision.GarminKeyEntry,
            PhoneRoutes.decide("POST", "/api/v1/auth/garmin/personal-key/test"),
        )
    }

    @Test
    fun `the phone has no password and no developer key`() {
        assertEquals(
            Decision.GarminLogin,
            PhoneRoutes.decide("POST", "/api/v1/auth/garmin/login"),
        )
        assertEquals(
            Decision.GarminLogin,
            PhoneRoutes.decide("POST", "/api/v1/auth/garmin/mfa"),
        )
        assertEquals(
            Decision.GarminDeveloperKey,
            PhoneRoutes.decide("POST", "/api/v1/auth/garmin/developer-key"),
        )
        assertEquals(
            Decision.GarminDeveloperKey,
            PhoneRoutes.decide("POST", "/api/v1/auth/garmin/exchange"),
        )
    }

    @Test
    fun `a pull is a pull`() {
        assertEquals(
            Decision.GarminPull,
            PhoneRoutes.decide("POST", "/api/v1/sync/run"),
        )
    }

    @Test
    fun `everything else goes to the core`() {
        assertEquals(
            Decision.Forward,
            PhoneRoutes.decide("GET", "/api/v1/activities"),
        )
        assertEquals(
            Decision.Forward,
            PhoneRoutes.decide("POST", "/api/v1/auth/garmin/fit-import"),
        )
        assertEquals(
            Decision.Forward,
            PhoneRoutes.decide("GET", "/api/v1/auth/garmin/personal-key"),
        )
        assertEquals(
            Decision.Forward,
            PhoneRoutes.decide("PATCH", "/api/v1/activities/1"),
        )
    }

    @Test
    fun `the method is part of the decision`() {
        // The same path, read and written, are different things.
        assertEquals(
            Decision.Forward,
            PhoneRoutes.decide("GET", "/api/v1/sync/run"),
        )
        assertEquals(
            Decision.Forward,
            PhoneRoutes.decide("DELETE", "/api/v1/auth/garmin/personal-key"),
        )
    }

    @Test
    fun `a pull and a file import write real data`() {
        // The two routes that put the athlete's own history on the phone.
        assertTrue(
            PhoneRoutes.writesRealData(
                PhoneRoutes.decide("POST", "/api/v1/sync/run"),
                "/api/v1/sync/run",
            )
        )
        assertTrue(
            PhoneRoutes.writesRealData(
                PhoneRoutes.decide("POST", "/api/v1/auth/garmin/fit-import"),
                "/api/v1/auth/garmin/fit-import",
            )
        )
    }

    @Test
    fun `reading the phone's data does not write any`() {
        // The screens read: Today, Runs, Progress, Body. None of them
        // writes, so none of them needs a screen lock.
        for (path in listOf(
            "/api/v1/insights/today",
            "/api/v1/insights/progress",
            "/api/v1/insights/chips",
            "/api/v1/activities",
            "/api/v1/activities/1",
            "/api/v1/stats/summary",
            "/api/v1/physiology/latest",
            "/api/v1/physiology/sources",
            "/api/v1/auth/status",
            "/api/v1/sync/status",
        )) {
            val decision = PhoneRoutes.decide("GET", path)
            assertFalse(
                "reading $path must not count as writing real data",
                PhoneRoutes.writesRealData(decision, path),
            )
        }
    }

    @Test
    fun `the key and the developer key never count as real data`() {
        // The Garmin key entry opens the native screen, which is where the
        // key is typed; it writes no history, so the screen-lock rule does
        // not apply to it.
        for (path in listOf(
            "/api/v1/auth/garmin/personal-key",
            "/api/v1/auth/garmin/personal-key/test",
        )) {
            val decision = PhoneRoutes.decide("POST", path)
            assertEquals(Decision.GarminKeyEntry, decision)
            assertFalse(
                "the key entry must not count as writing real data",
                PhoneRoutes.writesRealData(decision, path),
            )
        }
    }
}
