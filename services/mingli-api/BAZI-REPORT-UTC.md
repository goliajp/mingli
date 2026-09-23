# UTC-aware Bazi report v2

`POST /api/bazi/report/utc` is a separate opt-in endpoint. `/api/bazi/report`, its v1 evidence shape, and `/api/bazi` remain unchanged. V2 identifies `vsop87d-iau1980-utc-v2`, schema version 2.

The chart's year, solar month and cycle calculation all use the same TT solar roots. Civil dates and late-Zi day/hour conventions remain unchanged. The birth input still has minutes, not seconds; the API does not accept or manufacture a user's leap-second birth input.

## Coordinates and elapsed time

`birth_jd_civil` and each root's `jd_civil` are ordinary Gregorian calendar coordinates with 86,400 coordinate seconds per day. They are **not UTC quasi-JD** and are not labeled UT1. `birth_jde_tt` and root `jde_tt` are TT.

V2 `interval_days` is the absolute TT difference between birth and the selected root. A minute spanning an inserted leap second can therefore represent 61 elapsed TT seconds. Subtracting the civil coordinates instead would lose that second. Derived ages retain the prior three-days-per-year rule and independent integer/two-decimal rounding.

## Fixed conversion data

`time_scale.kind` is one of:

- `historical_ut_approx`: pre-1960 historical UT approximation with Espenak–Meeus delta-T; this is not reconstructed historical UTC
- `utc_drift_table`: the 14 pre-1972 piecewise linear UTC adjustments, including the 1960 segment, from ERFA v2.0.1
- `utc_leap_table`: 1972 onward fixed TAI−UTC steps, with TT−TAI = 32.184 seconds

The numerical table and retained originals are in `crates/mingli-bazi/src/report_utc_data.rs` and `data/utc-v2/`. ERFA's original copyright notice and license are retained. The IERS leap list states that it is in the public domain. Source URLs:

- [ERFA v2.0.1 dat.c](https://raw.githubusercontent.com/liberfa/erfa/v2.0.1/src/dat.c)
- [ERFA license](https://raw.githubusercontent.com/liberfa/erfa/v2.0.1/LICENSE)
- [IERS leap-seconds.list](https://hpiers.obspm.fr/iers/bul/bulc/ntp/leap-seconds.list)
- [IERS Bulletin C 72](https://hpiers.obspm.fr/iers/bul/bulc/bulletinc.72)

Hashes, source table identity, announcement publication, last offset effective date and file maintenance expiration are separately recorded in every v2 basis. Their roles must not be conflated:

| Date | Meaning |
|---|---|
| 2017-01-01 | Last effective TAI−UTC step to 37 seconds |
| 2026-07-06 | Bulletin C72 publication |
| 2027-06-28 | Downloaded leap-list maintenance expiration |
| 2027-07-01, exclusive | This contract's coverage cutoff |

C72 confirms that no leap second is introduced at the end of December 2026 and says leap seconds can occur at the end of June or December. The cutoff is an explicit inference from those statements: ordinary instants before the next possible end-June adjustment are covered. It is not a claim that the June 2027 adjustment, its inserted second, or July's offset has already been announced.

Coverage is `[1899-01-01,2027-07-01)` for required calculations; user birth years still begin at 1900. Birth **and all required roots** must be covered, so a June 2027 birth can already be rejected if its next jie is in July. V1 continues to support its existing approximate 1900–2100 calculation range.

## Root solving and failures

Solar roots are solved in TT without restricting trial guesses to the civil data window. Only the solved roots are inverted through the fixed historical/UTC segments. Unsupported coverage, leap/offset gaps and ambiguous backward adjustments return typed domain errors and HTTP 400; no root is clamped into a valid-looking result.

At an exact drift-segment effective midnight, inverse floating-point subtraction can lose one JD unit in the last place. The inverse anchors only exact equality with the segment start's forward TT value. This preserves real leap gaps and negative-adjustment overlaps.

The v2 basis contains `time_scale_policy`, `birth_time_scale`, complete `li_chun`, `previous_jie`, `next_jie`, and `interval_scale: "tt_elapsed"`. Per-root evidence records the exact table segment and applied TT offset. These are calculation evidence, not user-facing fortune claims or a promise that modelled solar positions equal observations.

## Recorded-minute alternatives

`POST /api/bazi/report/utc/minute` accepts the same clock-time request but returns
`schema_version: 1`, `model_id: vsop87d-iau1980-utc-minute-v1`. This is a separate
envelope version; each nested report retains UTC cycle evidence schema 2.

`recorded_input` preserves the supplied minute. `minute` and each candidate's
`interval` contain `start` / `end` UTC instants, `[start,end)`, explicit inclusion
booleans and physical `elapsed_tt_seconds`. Solar `boundaries` strictly inside
the recorded minute split candidates into `before` and `after`; otherwise there
is a `single` candidate. Every candidate has a full freshly computed `report`,
an interior civil-midpoint `representative`, and an unrounded `start_age_range`
with both endpoint-inclusion flags. The representative is an assumption for
that interval, not inferred birth seconds. Nested chart input stays at the
recorded minute while cycle evidence birth coordinates identify the actual
representative. Day/hour remain those of the recorded local minute; year,
month, direction, ten cycles and all chart-derived fields are independently
recomputed on each side. At Li Chun the changed year can reverse cycle direction.

Ranges use physical TT elapsed time: the 2016-12-31 UTC last minute spans 61
seconds. The endpoint never invents an ordinary civil second 60. Negative
historic UTC steps or ambiguous representative instants return HTTP 400, as do
unsupported coverage, unavailable adjacent roots and true-solar input. A root
exactly at the minute start belongs to that minute; an end root is excluded.
Unrepresentably narrow floating-point intervals return a typed error rather
than an empty or fabricated candidate. The service does not publish a textual
reading or claim that a complete product report is ready.
