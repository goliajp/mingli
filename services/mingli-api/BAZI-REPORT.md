# Bazi report calculation evidence v1

`POST /api/bazi/report` accepts the existing birth request shape. It requires a calculation gender (`male` or `female`, including the existing DTO aliases), a valid 1900–2100 birth date/time and timezone, and clock time (`true_solar_time` omitted or false). A true value is rejected even without longitude. Default Bazi school remains late Zi hour and Li Chun year boundary. Like `/api/bazi`, the endpoint does not select schools from the generic request's `schools` field.

The response is `{chart, cycle_basis}`. `chart` has exactly the existing `/api/bazi` clock-time response shape, with no extra fields. Its year/month pillars and cycle ages use the new high-order model and can differ from the legacy endpoint near a jie. The old endpoint and its optional-gender behavior are unchanged. The new response is stamped with the same compiled `x-mingli-build-id` header as other endpoints.

`cycle_basis` fields:

| Field | Meaning |
|---|---|
| `schema_version` | `1` |
| `model_id` | `vsop87d-iau1980-delta-t-v1` |
| `birth_jd_ut` | Birth instant, UT Julian day |
| `birth_jde_tt` | Same instant converted to TT with the engine delta-T model |
| `birth_longitude_deg` | Apparent solar longitude used by the chart |
| `previous_jie` / `next_jie` | `{target_longitude_deg,jd_ut}`; target normalized to `[0,360)`, jie targets are 15° modulo 30° |
| `selected` | `previous` for reverse or `next` for forward cycles |
| `interval_days` | Birth-to-selected-jie interval from the same solved roots |
| `unrounded_start_age_years` | `interval_days / 3`, before rounding |
| `days_per_year` | `3` |

Both adjacent roots and the unrounded interval are captured in the same function invocation that creates `chart.dayun`. They are not independently recalculated from displayed ages. The first integer age is `round(unrounded_start_age_years)`; `start_age_years` is separately rounded to two decimal places. Rounding the published decimal again can disagree near half-year boundaries.

This is transparent evidence of the explicitly named model, not an independent astronomical oracle or a promise of minute-accurate jie times. In particular, a birth very near a model's jie boundary can receive a different month/year or selected jie under another astronomical model. Consumers must establish their own acceptable accuracy and boundary policy before using this endpoint for a professional report. The endpoint does not infer fate, event outcomes or a precise civil date of changing luck.

Verification includes equality of the old endpoint with a frozen pre-extension real chart, compatibility of the new response shape, and an explicit 2013-05-05 boundary case where the new model changes the month and cycle together. Missing gender, true-solar and invalid birth inputs are rejected. The retained legacy evidence function is checked at each of 2,412 jie in 1900–2100 at neighboring civil minutes in UTC−12, UTC+05:45 and UTC+14 for both genders. The high-order report model has a separate full-year boundary matrix described below. These checks establish internal consistency with each named model, not astronomical accuracy.

## High-order report model

The report-only `report-vsop` feature uses pinned `vsop87=3.0.0` full Earth VSOP87D coefficients (equinox of date), solar FK5 longitude correction −0.09033 arcseconds, `astro=2.0.0` IAU1980 nutation and approximate aberration −20.4898 arcseconds divided by Earth–Sun distance in AU. It does not precess the already date-referenced VSOP87D coordinates a second time. Definitions follow [Astronomia's solar reference implementation](https://github.com/commenthol/astronomia/blob/v4.2.0/src/solar.js).

All three solar-sensitive decisions—Li Chun year, solar month and neighboring jie for cycles—use the same new model. `compute_report` remains the legacy Meeus evidence API; `compute_report_vsop` supplies the new endpoint. No old chart contract or historical record is silently reinterpreted.

The report UT→TT conversion supports the adjacent 1899–2101 calculation interval needed by local 1900–2100 births. It adds the Espenak–Meeus 1860–1900 delta-T polynomial for 1899, retaining the existing later segments and decimal-year mapping `2000+(JD−2451545)/365.25`. This extrapolation model is not observed UT1 and is not independently validated by comparing TT roots.

A source review found `astro`'s lunar argument F cubic term has the opposite sign to Astronomia/Meeus's expression. The pinned implementation remains explicit; cross-implementation comparisons measure its combined effect. The solar-specific FK5 correction omits the general planet latitude-dependent longitude term; the tests measure that omitted term over all 2,412 jie rather than claiming it is identically zero.

The dependencies already exist in the full API's planetary ephemeris dependency graph. New direct dependencies are optional: default standalone Bazi and default app/WASM builds do not gain the report-only feature. No runtime coefficient download is needed.

The maximum omitted general FK5 latitude term across the 2,412 tested high-order jie roots is **0.000000318760 arcseconds**. This is a sampled contribution, not an all-time mathematical bound. All 9,648 high-order adjacent-minute/direction cases pass year/month/root consistency checks; an additional 72 cases cover timezone extremes, civil year boundaries and 23:00. The legacy model’s separate 28,944-case consistency suite remains passing.
