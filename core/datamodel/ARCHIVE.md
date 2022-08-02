# Archive

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
