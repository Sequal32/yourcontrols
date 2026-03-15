class YourControlsNetwork {
	constructor(onMessageCallback, connectedCallback, disconnectedCallback, canConnect) {
		this.socket = null;
		this.socketConnected = false;
		this.instrumentName = "";

		this.onMessageCallback = onMessageCallback ? onMessageCallback : () => { };
		this.connectedCallback = connectedCallback ? connectedCallback : () => { };
		this.disconnectedCallback = disconnectedCallback ? disconnectedCallback : () => { };
		this.canConnect = canConnect ? canConnect : () => false;
	}

	connectWebsocket() {
		if (this.socket !== null) {
			this.socket.close();
		}

		this.socket = new WebSocket('ws://127.0.0.1:7780');

		this.socket.addEventListener('open', this.onConnected.bind(this));
		this.socket.addEventListener('close', this.onConnectionLost.bind(this));
		this.socket.addEventListener('error', this.onConnectionError.bind(this));
		this.socket.addEventListener('message', this.onSocketMessage.bind(this));
	}

	onConnectionLost() {
		this.socket = null;
		this.socketConnected = false;
		this.disconnectedCallback();
	}

	onConnectionError() {
		this.socket.close();
	}

	isYourControlsRunning() {
		return SimVar.GetSimVarValue("L:YourControlsServerRunning", "boolean") == true;
	}

	startAttemptConnection(instrumentName) {
		this.instrumentName = instrumentName;
		setInterval(this.attemptConnection.bind(this), 4000);
	}

	attemptConnection() {
		if (this.socketConnected || !this.canConnect() || !this.isYourControlsRunning()) {
			return;
		}
		this.connectWebsocket();
	}

	sendObjectAsJSON(message) {
		if (this.socket === null || this.socket.readyState != 1) {
			return;
		}
		this.socket.send(JSON.stringify(message));
	}

	sendInteractionEvent(eventName) {
		const sendObjectAsJSON = this.sendObjectAsJSON.bind(this);
		// Allow time for other vars to sync before H event.
		setTimeout(() => {
			sendObjectAsJSON({
				type: "interaction",
				name: eventName
			});
		}, 100);
	}

	sendInputEvent(elementId, value) {
		this.sendObjectAsJSON({
			type: "input",
			id: elementId,
			value: value
		});
	}

	onConnected() {
		console.log("YourControls websocket connected.");
		this.socketConnected = true;

		this.sendObjectAsJSON({
			type: "handshake",
			name: this.instrumentName,
		});

		this.connectedCallback();
	}

	onSocketMessage(event) {
		let data = JSON.parse(event.data);
		this.onMessageCallback(data);
	}
}