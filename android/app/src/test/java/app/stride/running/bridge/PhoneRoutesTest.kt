package app.stride.running.bridge

import app.stride.running.bridge.PhoneRoutes.Decision
import org.junit.Assert.assertEquals
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
}
