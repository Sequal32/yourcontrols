// vars[0]: current value
// sets[0]: set event name

let current_value = vars.get(0)

if incoming_value == current_value {return;}

sets.set(0, incoming_value)