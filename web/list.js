class ConnectionList {
    constructor(html_list) {
        this.object = html_list
        this.list = {}
    }

    update() {
        for (var key in this.list) {
            this.list[key].setButtonsVisibility(has_control, is_client)
        }
    }

    clear() {
        this.lastInControl = null
        for (var key in this.list) {
            this.remove(key)
        }
    }

    hideStatusText() {
        for (var key in this.list) {
            this.list[key].hideStatus()
        }
    }

    add(name) {
        var listItem = document.createElement("li")
        listItem.className = "list-group-item themed"
        listItem.innerText = name

        var controlButton = document.createElement("button")
        controlButton.className = "btn btn-outline-primary btn-sm entry-button"
        controlButton.type = "button"
        controlButton.innerHTML = "Give Control"

        var observeButton = document.createElement("button")
        observeButton.className = "btn btn-outline-secondary btn-sm entry-button"
        observeButton.type = "button"
        observeButton.innerHTML = "Observer"

        var statusText = document.createElement("p")
        statusText.className = "entry-button"
        statusText.innerHTML = "In Control"
        statusText.hidden = true
        // Add as childs
        listItem.append(controlButton, observeButton, statusText)
        this.object.append(listItem)
        // listItem as class
        let listItemObject = new ConnectionListItem(listItem, name)
        this.list[name] = listItemObject

        listItemObject.setButtonsVisibility(has_control, is_client)
    }

    setInControl(name) {
        if (this.lastInControl) {
            this.list[this.lastInControl].setInControl(false)
        }
        this.list[name].setInControl(true)
        this.lastInControl = name
    }

    setObserver(name, observing) {
        this.list[name].setObserver(observing)
    }

    remove(name) {
        this.object.removeChild(this.list[name].object)
        delete this.list[name]
    }

    hide() {
        this.object.hidden = true
    }

    show() {
        this.object.hidden = false
    }
}

class ConnectionListItem {
    constructor(htmlObject, name) {
        this.object = htmlObject
        this.controlButton = htmlObject.children[0]
        this.observeButton = htmlObject.children[1]
        this.statusText = htmlObject.children[2]
        this.name = name

        this.is_observer = false

        this.controlButton.onclick = this.controlButtonClicked.bind(this)
        this.observeButton.onclick = this.observeButtonClicked.bind(this)
    }

    observeButtonClicked() {
        var removing = this.observeButton.classList.contains("btn-outline-secondary")
        this.observeButton.classList.toggle("btn-outline-secondary")
        this.observeButton.classList.toggle("btn-secondary")

        this.is_observer = !this.is_observer
        this.controlButton.hidden = this.is_observer

        invoke({
            type: "setObserver",
            is_observer: removing,
            target: this.name
        })
    }

    controlButtonClicked() {
        this.controlButton.hidden = true
        this.observeButton.hidden = true
        invoke({
            type: "transferControl",
            target: this.name
        })
    }

    setInControl(inControl) {
        this.statusText.innerHTML = "In Control"
        this.statusText.classList.toggle("entry-text-observe", !inControl)
        this.statusText.classList.toggle("entry-text-in-control", inControl)
        this.statusText.hidden = !inControl
    }

    setObserver(observing) {
        this.is_observer = observing
        this.statusText.innerHTML = "Observing"
        this.statusText.classList.toggle("entry-text-observe", observing)
        this.statusText.classList.toggle("entry-text-in-control", !observing)
        this.statusText.hidden = !observing
    }

    setButtonsVisibility(hasControl, isClient) {
        this.controlButton.hidden = this.is_observer || (!hasControl && !this.is_observer)
        this.observeButton.hidden = isClient || this.controlButton.hidden
    }

    hideStatus() {
        this.statusText.hidden = true
    }
}