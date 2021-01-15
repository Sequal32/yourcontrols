function ConnectionList(html_list) {
    this.object = html_list
    this.list = {}
}

ConnectionList.prototype.update = function() {
    for (var key in this.list) {
        this.list[key].setButtonsVisibility(has_control)
    }
}

ConnectionList.prototype.clear = function() {
    this.lastInControl = null
    for (var key in this.list) {
        this.remove(key)
    }
}

ConnectionList.prototype.hideStatusText = function() {
    for (var key in this.list) {
        this.list[key].hideStatus()
    }
}

ConnectionList.prototype.add = function(name) {
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
    listItem.appendChild(controlButton)
    listItem.appendChild(observeButton)
    listItem.appendChild(statusText)
    this.object.appendChild(listItem)
    // listItem as class
    let listItemObject = new ConnectionListItem(listItem, name)
    this.list[name] = listItemObject

    listItemObject.setButtonsVisibility(has_control)
    if (!is_client) {listItemObject.observeButtonClicked()}
}

ConnectionList.prototype.setInControl = function(name) {
    if (this.lastInControl) {
        this.list[this.lastInControl].setInControl(false)
    }
    this.list[name].setInControl(true)
    this.lastInControl = name
}

ConnectionList.prototype.setObserver = function(name, observing) {
    this.list[name].setObserver(observing)
}

ConnectionList.prototype.remove = function(name) {
    if (this.lastInControl == name) {
        this.lastInControl = null
    }
    this.object.removeChild(this.list[name].object)
    delete this.list[name]
}

ConnectionList.prototype.hide = function() {
    this.object.hidden = true
}

ConnectionList.prototype.show = function() {
    this.object.hidden = false
}

function ConnectionListItem(htmlObject, name) {
    this.object = htmlObject
    this.controlButton = htmlObject.children[0]
    this.observeButton = htmlObject.children[1]
    this.statusText = htmlObject.children[2]
    this.name = name

    this.is_observer = false

    this.controlButton.onclick = this.controlButtonClicked.bind(this)
    this.observeButton.onclick = this.observeButtonClicked.bind(this)
}


ConnectionListItem.prototype.observeButtonClicked = function() {
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

ConnectionListItem.prototype.controlButtonClicked = function() {
    this.controlButton.hidden = true
    this.observeButton.hidden = true
    invoke({
        type: "transferControl",
        target: this.name
    })
}

ConnectionListItem.prototype.setInControl = function(inControl) {
    this.statusText.innerHTML = "In Control"
    this.statusText.classList.toggle("entry-text-observe", !inControl)
    this.statusText.classList.toggle("entry-text-in-control", inControl)
    this.statusText.hidden = !inControl
}

ConnectionListItem.prototype.setObserver = function(observing) {
    this.is_observer = observing
    this.statusText.innerHTML = "Observing"
    this.statusText.classList.toggle("entry-text-observe", observing)
    this.statusText.classList.toggle("entry-text-in-control", !observing)
    this.statusText.hidden = !observing || !is_client
}

ConnectionListItem.prototype.setButtonsVisibility = function(hasControl) {
    this.controlButton.hidden = this.is_observer || (!hasControl && !this.is_observer)
    this.observeButton.hidden = is_client || this.controlButton.hidden
}

ConnectionListItem.prototype.hideStatus = function() {
    this.statusText.hidden = true
}