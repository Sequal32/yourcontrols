// vars[0]: current switch value
// sets[0]: on_event_name/event_name, sets[1]: off_event_name

let is_switch_on = vars.get(0);
let only_on = params[0]; // Should be triggered if and only if the incoming value is the "on" state

if incoming_value == is_switch_on {return;}
if !incoming_value && only_on {return;}

let use_off_event = sets.exists(1);

if use_off_event && !incoming_value {
    sets.set(1);
} else {
    sets.set(0);
}