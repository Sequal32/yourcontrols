const clickEvents = ["mousedown", "mouseup", "click"].map(eventType => {
	let evt = new MouseEvent(eventType, { cancelable: true, bubbles: true });
	evt.YC = true;
	return evt;
});

const nativeInputSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;

class YourControlsHTMLTrigger {
	static setInput(element, value) {
		nativeInputSetter.call(element, value);

		const inputEvent = new Event('input', { bubbles: true });
		inputEvent.YC = true;

		element.dispatchEvent(inputEvent);
	}

	static setPanel(eventName, instrumentName) {
		const split = eventName.indexOf("#");
		const targetInstrumentName = eventName.substring(0, split);

		if (targetInstrumentName != instrumentName) {
			return;
		}

		const buttonName = eventName.substring(split + 1);
		const button = document.getElementById(buttonName);

		clickEvents.forEach(evt => {
			button.dispatchEvent(evt);
		});
	}
}