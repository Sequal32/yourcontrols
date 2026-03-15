var handler = null;

class YourControlsHandler {
	constructor(instrumentName, isTouch) {
		this.isTouch = isTouch;
		this.instrumentName = instrumentName;

		this.net = new YourControlsNetwork(
			this.onMessage.bind(this),
			this.onConnected.bind(this),
			this.onDisconnected.bind(this),
			() => this.canProcess() || this.isTouch
		);
		this.events = new YourControlsHTMLEvents(
			this.onButton.bind(this),
			this.onInput.bind(this)
		);

		this.panelId = Math.floor(Math.random() * 100000);

		SimVar.SetSimVarValue("L:YourControlsPanelId", "number", this.panelId); // Only one panel should fire event

		this.start();
	}

	canProcess() {
		return SimVar.GetSimVarValue("L:YourControlsPanelId", "number") == this.panelId;
	}

	start() {
		setTimeout(this.startCall.bind(this), 3000);
	}

	startCall() {
		this.net.startAttemptConnection(this.instrumentName);
	}

	onMessage(data) {
		switch (data.type) {
			case "input": {
				YourControlsHTMLTrigger.setInput(document.getElementById(data.id), data.value);
				break;
			}
			case "time": {
				if (this.canProcess()) {
					SimVar.SetSimVarValue("K:ZULU_YEAR_SET", "number", data.year);
					SimVar.SetSimVarValue("K:ZULU_DAY_SET", "number", data.day);
					SimVar.SetSimVarValue("K:ZULU_HOURS_SET", "number", data.hour);
					SimVar.SetSimVarValue("K:ZULU_MINUTES_SET", "number", data.minute);
				}
				break;
			}
			case "requestTime": {
				if (!this.canProcess()) break;

				const totalSeconds = SimVar.GetSimVarValue("E:ZULU TIME", "seconds");
				const minute = Math.floor((totalSeconds % 3600) / 60);
				const hour = Math.floor(totalSeconds / 3600) % 24;

				this.net.sendObjectAsJSON({
					type: "time",
					minute: minute,
					hour: hour,
					day: SimVar.GetSimVarValue("E:ZULU DAY OF YEAR", "number"),
					year: SimVar.GetSimVarValue("E:ZULU YEAR", "number")
				});
				break;
			}
		}
	}

	onConnected() {
		if (this.instrumentName.includes("WT21_FMC")) return;
		if (this.isTouch) {
			this.events.bindEvents();
			this.events.startDocumentListener();
		}
	}

	onDisconnected() {
		this.events.clear();
	}

	onInput(elementId, value) {
		this.net.sendInputEvent(elementId, value);
	}

	onButton(elementId) {
		this.net.sendInteractionEvent("H:YCB" + this.instrumentName + "#" + elementId);
	}

	processInteractionEvent(args) {
		if (args[0].startsWith("YCB")) {
			YourControlsHTMLTrigger.setPanel(args[0].substring(3), this.instrumentName);
			return false;
		} else if (args[0].startsWith("YCH")) {
			args[0] = args[0].substring(3);
		} else {
			if (this.canProcess()) { // Only one panel should send interaction button events
				this.net.sendInteractionEvent("H:YCH" + args[0]);
			}
		}
		return true;
	}
}