// vars[0]: current value
// sets[0]: set event name

const current_value = vars[0].get();

if incoming_value == current_value {return;}

sets[0].set(incoming_value);