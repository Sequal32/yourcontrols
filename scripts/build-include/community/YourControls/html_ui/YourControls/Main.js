var handler = null

class YourControlsHandler {
    constructor(instrumentName, isTouch) {
        this.isTouch = isTouch
        this.instrumentName = instrumentName

        this.net = new YourControlsNetwork(this.onMessage.bind(this), this.onConnected.bind(this), this.onDisconnected.bind(this), () => this.canProcess() || this.isTouch)
        this.events = new YourControlsHTMLEvents(this.onButton.bind(this), this.onInput.bind(this))

        this.panelId = Math.floor(Math.random() * 100000)
        // Only one panel should fire event
        SimVar.SetSimVarValue("L:YourControlsPanelId", "Number", this.panelId)

        this.start()
    }

    canProcess() {
        return SimVar.GetSimVarValue("L:YourControlsPanelId", "Number") == this.panelId
    }

    start() {
        setTimeout(this.startCall.bind(this), 3000)
    }

    startCall() {
        this.net.startAttemptConnection(this.instrumentName)
    }

    onMessage(data) {
        switch (data.type) {
            case "input": {
                YourControlsHTMLTrigger.setInput(document.getElementById(data.id), data.value)
                break;
            }
            case "time": {
                if (this.canProcess()) {
                    SimVar.SetSimVarValue("K:ZULU_HOURS_SET", "Number", data.hour)
                    SimVar.SetSimVarValue("K:ZULU_MINUTES_SET", "Number", data.minute)
                    SimVar.SetSimVarValue("K:ZULU_DAY_SET", "Number", data.day)
                    SimVar.SetSimVarValue("K:ZULU_YEAR_SET", "Number", data.year)
                }
                break;
            }
            case "requestTime": {
                if (!this.canProcess()) {break}
                const hour = SimVar.GetSimVarValue("E:ZULU TIME", "Hours")
                const minute = Math.ceil((hour % 1) * 60 )

                this.net.sendObjectAsJSON({
                    type: "time",
                    minute: minute,
                    hour: Math.floor(hour),
                    day: SimVar.GetSimVarValue("E:ZULU DAY OF YEAR", "Number"),
                    year: SimVar.GetSimVarValue("E:ZULU YEAR", "Number")
                })
                break;
            }
        }
    }

    onConnected() {
        if (this.isTouch) {
            this.events.bindEvents()
			this.events.startDocumentListener()
        }
    }

    onDisconnected() {
        this.events.clear()
    }

    onInput(elementId, value) {
        this.net.sendInputEvent(elementId, value)
    }

    onButton(elementId) {
        this.net.sendInteractionEvent("H:YCB" + this.instrumentName + "#" + elementId)
    }

    processInteractionEvent(args) {
        // Panel event
        if (args[0].startsWith("YCB")) {

            YourControlsHTMLTrigger.setPanel(args[0].substring(3), this.instrumentName)
            return false

        } else if (args[0].startsWith("YCH")) {

            args[0] = args[0].substring(3)

        } else {
            if (this.canProcess()) {  // Only one gauge should send interaction button events  
                this.net.sendInteractionEvent("H:YCH" + args[0])
            }
        }
        return true
    }
}