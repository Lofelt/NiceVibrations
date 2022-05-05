- [SDK Data Model](#sdk-data-model)
  - [Version 1.0(.0)](#version-100)
  - [Version 0.2(.0)](#version-020)
    - [VIJ Root](#vij-root)
    - [VIJ Non Haptic Data](#vij-non-haptic-data)
      - [Metadata](#metadata)
      - [Variation](#variation)
    - [VIJ Haptic Data](#vij-haptic-data)
      - [Voices](#voices)
    - [Validation Rules](#validation-rules)
    - [JSON schema](#json-schema)
- [Benchmarking](#benchmarking)
  - [Criterion](#criterion)
  - [Benchmark Data](#benchmark-data)
  - [Benches](#benches)
  - [Running Benchmarks](#running-benchmarks)
    - [Timing](#timing)
  - [Reports](#reports)

#  SDK Data Model
The Lofelt Data Model describes vibrotactile content derived from an audio signal source.

This crate contains the schema of the Lofelt SDK Data model as well as related functions, conversions, and versioning.
Currently, it provides JSON serialization and deserialization of Lofelt Data, upgrade, as well as conversion to  platform-specific haptic data.

## Version 1.0(.0)

![](./media/v1-diag.svg)

Currently this version is a WIP and can be discussed in a [Lofelt Confluence page](https://lofelt.atlassian.net/wiki/spaces/PD/pages/196116488/Haptic+File+Format).

## Version 0.2(.0)


### VIJ Root

| **Field**  | **Type**        | **Description**                   | **Required**            |
| ---------- | --------------- | --------------------------------- | ----------------------- |
| Metadata   | Structure array | Contextual data for the VIJ data. | ![](./media/image1.png) |
| Variations | Structure array | ?                                 | ![](./media/image2.png) |
| Voices     | Structure Array | Contains haptic data.             | ![](./media/image1.png) |

### VIJ Non Haptic Data

#### Metadata

| **Field** | **Type** | **Description**                           | **Required**            | **Default** |
| --------- | -------- | ----------------------------------------- | ----------------------- | ----------- |
| Editor    | String   | Name of tool generating the file          | ![](./media/image2.png) | ““          |
| Format    | String   | Format of DSP analysis                    | ![](./media/image2.png) | ““          |
| Duration  | Float    | Duration of the haptic content in seconds | ![](./media/image1.png) | 0.0         |

#### Variation

| **Field**  | **Type** | **Range** | **Description**                     | **Required**            | **Default** |
| ---------- | -------- | --------- | ----------------------------------- | ----------------------- | ----------- |
| Total gain | Float    | 0.0 – 1.0 | Global gain of the amplitude values | ![](./media/image2.png) | 1.0         |
| Partials   | Float    |           | ?                                   | ![](./media/image2.png) | Empty       |

### VIJ Haptic Data

#### Voices

| **Field**  | **Sub-field** | **Type**        | **Range**         | **Description**                                                                                                                                                        | **Required**                            | **Default** |
| ---------- | ------------- | --------------- | ----------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------- | ----------- |
| Envelopes  |               | Structure Array |                   | Contains multiple arrays modulating each band, where the first array correspondents to an envelope of Amplitude, and the second array to Frequency Modulation.         | ![](./media/image1.png)At least 1 array |             |
|            | Time          | double          | 0.0 - ≈1.798E+308 | Point in time of envelope point                                                                                                                                        | ![](./media/image1.png)                 |             |
|            | Amplitude     | double          | 0.0 – 1.0         | Amplitude value of envelope point                                                                                                                                      | ![](./media/image1.png)                 |             |
| Bands      | ?             |                 |                   | ?                                                                                                                                                                      | ![](./media/image2.png)                 |             |
| Transients |               | Structure Array |                   | Contains multiple arrays describing transient events, where the first array correspondents to their Amplitude values in time, and the second array to their Frequency. | ![](./media/image2.png)                 |             |
|            | Time          | double          | 0.0 - ≈1.798E+308 | Point in time of transient                                                                                                                                             | ![](./media/image1.png)                 |             |
|            | Amplitude     | double          | 0.0 - 1.0         | Amplitude value of transient                                                                                                                                           | ![](./media/image1.png)                 |             |

### Validation Rules

- Voices Envelopes':
  - Needs to have breakpoints
  - Needs to have the first envelope array (corresponding to amplitude envelope) not empty
  - Breakpoint `time` needs to be consecutive
  - Breakpoints `amplitude` needs to be in its *Range*
 - Transients
   - Breakpoint `time` needs to be consecutive
   - Breakpoints `amplitude` needs to be in its *Range*
   - The transient amplitude and frequency arrays need to have matching `time` values
   - The transient amplitude and frequency arrays need to have the same array length



### JSON schema

````
{
  "metadata": {
    "editor": string,
    "format": string,
    "duration": float
  },
  "variation": {
    "total gain": float,
    "partials": float
  },
  "voices": {
    "envelopes": [
      [
        {
          "time": float,
          "amplitude": float
        },
		...
      ],
      [
        {
          "time": float,
          "amplitude": float
        },
		...
      ]
    ],
    "bands": [],
    "transients": [
      [
        {
          "time": float,
          "amplitude": float
        },
		...
      ],
      [
        {
          "time": float,
          "amplitude": float
        },
		...
      ]
    ]
  }
}



````

# Benchmarking
## Criterion

Benchmarking is using the [Criterion crate, v0.3](https://bheisler.github.io/criterion.rs/book/) for running benches.
It is added as a "Dev Dependency" and does not get pulled into other crates.
We should probably add it separately per crate.


## Benchmark Data

Data to benchmark has been chunked into:
- 'v1' (as of 30 June 2020)
- v0

for the following lengths of audio:
- 0.5s
- 1s
- 10
- 60s
- 120s

 Samples were chopped in Ableton Live from "wildfire.mp3" then run through Studio Desktop
 with the settings:
 - normalise enabled
 - bptransients enable

NOTE: this data will need to be recreated every time our data model changes


## Benches

All benchmarks are currently sitting as one big group in ` ` ` benches/datamodel_benches` ` `

## Running Benchmarks

Currently we are benchmarking all deserialisation, validation and conversion functions for v1, v0 and ahap.
This tool can be used to look for regressions in all of these areas and further benches can be added for other core data manipulation as we go.

Run:

```` cargo bench ````

from the datamodel crate *before* making any changes to the code.
This will generate your first set of data (baseline).

Make changes you want to make, and run the command again. You should be able to see if any regressions are introduced.

### Timing
On occasion your benchmark may not run long enough. You will get a message on the console that tells you how long you should run it for.
To adjust this timing change the Duration in the benchmark.

````g.measurement_time(time::Duration::from_secs(NEWTIMEINSECONDS));````


## Reports

*Note:* you need to install gnuplot to generate reports.

````  brew install gnuplot````

Reports can be found as datamodel/target/.criterion/report/index.html


