# Theory
*The best way to become familiar with this is to look through the configuration files already provided, and see if you can add more variables/events to synchronize more.*

*SDK links are substituted for P3D links as they hold basically the same content. If you'd like an updated list of events/variables, download the MSFS SDK through the DevTools in MSFS*

## YAML files
YAML, similar to Python relies of the identation of files in order to parse its contents. Keep this in mind while writing one.

`-` are used to denote lists.
`somestring:` words followed by a colon denote a dictionary.

In the configuration files case, `master:` or `shared:` are used to denote that the entries following those are restricted to either the person in control, or can be synced both ways respectively.

## What is a variable?
Take a look through the list of defined variables [here](http://www.prepar3d.com/SDKv2/LearningCenter/utilities/variables/simulation_variables.html) in the SDK. These are known as **aircraft variables**, or `A:` variables. We can get what variable a specific aircraft uses by looking through the source code, or by constantly retrieving it and detecting changes.

There are also variables that are defined aircraft to aircraft, which are also known as **local variables** or `L:` variables. These are commonly found through looking through the source code of an aircraft.

## How do we put this into the config file?
Let's take `PLANE LATITUDE` as an example. In `modules/physics.yaml`, we have the following entry:
```yaml
type: var
var_name: PLANE LATITUDE
var_units: Degrees
var_type: f64
interpolate:
    overshoot: 10.0
```
`type: var` tells the application to treat this as simply a get/set variable. Once it detects a change in this variable, it'll notify the clients to update it.

`var_name: PLANE LATITUDE` should be self-explanatory. This is the variable we want to read.

`var_units: Degrees` tells SimConnect what kind of units to send the variable in. Notice how in the SDK, it displays a possible units as Radians, but SimConnect can also send the latitude in degrees instead.

`var_type: f64` tells the application what kind of datatype to store the value as. If your variable should have numbers after the decimal place (I.E 5.02 instead of 5), then this should be set to f64. Otherwise, bool (1 or 0) or i32 (5, -5, 0) should be used.

`interpolate:` tells the application that it should smoothy transition from the last received value to the next. For example if our current position is at 5.0, but we received a value of 10.0, we use the time in between to guess that we should be at 7.0.

## What is an event?
Notice how `PLANE LATITUDE` settable column has a Y in it compared to `LIGHT NAV`. This means we can only read the value, but not update it. In this case, the simulator uses **events** to update this value. You can find a list of events here [here](http://www.prepar3d.com/SDKv2/LearningCenter/utilities/variables/event_ids.html)

## How do we put this into the config file?
Let's take `LIGHT NAV` for example from `modules/lights.yaml`.
```yaml
type: ToggleSwitch
var_name: A:LIGHT NAV
var_units: Bool
event_name: TOGGLE_NAV_LIGHTS
```
`type: ToggleSwitch` tells the application to treat this entry as a switch which can only *toggled*. For example, if it's on, the event has to switch it off. The application checks if the current switch position and the updated switch position is different, and then sends the toggle event. Otherwise it won't do anything.
`event_name`: this is the event name found in the list of events. Note this isn't `KEY_TOGGLE_NAV_LIGHTS`, rather simply `TOGGLE_NAV_LIGHTS`

## Including other modules
*Including* is simply loading the data from another file into the current file. This makes it easier to write an entry for nav lights, but share it across multiple aircraft.
`definitions/modules/lights.yaml` Note that this is relative to the .exe's directory.

# Reference
## Types

### ToggleSwitch
Used for a switch that only has an event which can only flip a its state without modifying it directly
```yaml
type: ToggleSwitch
var_name: A:PITOT HEAT
var_units: Bool
event_name: PITOT_HEAT_TOGGLE
```

### ToggleSwitchParam
Used for toggle events that require a *constant* to be passed along with it. This is usually represented by number followed by the event code: `1 (>A:ELECTRICAL MASTER BATTERY:1)`

```yaml
type: ToggleSwitchParam
var_name: A:ELECTRICAL MASTER BATTERY:1
var_units: bool
event_name: TOGGLE_MASTER_BATTERY
event_param: 1
```

### ToggleSwitchTwo
Used for switches that have an event associated with it's on position, and off position.
```yaml
type: TOGGLESWITCHTWO
var_name: A:ELT ACTIVATED
var_units: Bool
var_type: bool
off_event_name: ELT_OFF
on_event_name: ELT_ON
```

### SwitchOn
Used for switches that have an event that only sets its state to the ON position.
```yaml
type: SwitchOn
var_name: A:COM TRANSMIT:1
var_units: Bool
var_type: bool
event_name: COM1_TRANSMIT_SELECT
```

### NumIncrement
Used for dials that have events that increment/deincrement its value by a given amount and cannot be set directly.
```yaml
type: NumIncrement
var_name: A:NAV OBS:1
var_units: Degrees
var_type: i32
up_event_name: VOR1_OBI_INC
down_event_name: VOR1_OBI_DEC
increment_by: 1
```
### NumIncrementFloat
Similar to NumIncrement but for floating point values.
```yaml
type: NumIncrement
var_name: A:NAV OBS:2
var_units: Degrees
var_type: i32
up_event_name: VOR1_OBI_INC
down_event_name: VOR2_OBI_DEC
increment_by: 1
```

### NumSwap
Mainly used for radios that require a comm swap to the active frequency to satisfy the variable. It will call event_name, then call swap_event_name.
```yaml
type: NumSwap
var_name: ADF ACTIVE FREQUENCY:1 
var_units: Frequency ADF BCD32
var_type: i32
event_name: ADF_COMPLETE_SET # Sets standby
swap_event_name: ADF1_RADIO_SWAP # Swap to active
```

### Event
This will listen for the user to trigger the specified event, then broadcast it to other clients to trigger the same event. This should be used as a last ditch effort to get something to synchronize, as if an aircraft gets out of sync, this doesn't have a way to getting it back in sync.
```yaml
type: event
event_name: TOW_PLANE_REQUEST
```

## Var Types
### i32
Holds an integer.
### f64
Holds a floating point.
### bool
Holds a boolean value