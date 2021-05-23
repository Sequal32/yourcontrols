// vars[0]: current switch value
// sets[0]: on_event_name/event_name, sets[1]: off_event_name
const set_switch_on = incoming_value

const is_switch_on = vars[0].get();
const only_on = params[0]; // Should be triggered if and only if the incoming value is the "on" state

if set_switch_on == is_switch_on {return;}
if set_switch_on == false && only_on {return;}

const use_off_event = sets[1] != ();

if use_off_event && !set_switch_on {
    sets.set(1);
} else {
    sets.set(0);
}