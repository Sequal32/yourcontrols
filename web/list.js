class ConnectionList {
    constructor(html_list) {
        this.object = html_list
        this.list = {}
    }

    update() {
        for (var key in this.list) {
            console.log(has_control, is_client)
            this.list[key].setButtonsVisibility(has_control, is_client)
        }
    }

    add(name) {
        var listItem = document.createElement("li")
        listItem.className = "list-group-item"
        listItem.innerText = name

        var controlButton = document.createElement("button")
        controlButton.className = "btn btn-outline-primary btn-sm entry-button"
        controlButton.type = "button"
        controlButton.innerHTML = "Give Control"

        var observeButton = document.createElement("button")
        observeButton.className = "btn btn-outline-secondary btn-sm entry-button"
        observeButton.type = "button"
        observeButton.innerHTML = "Observer"
        // Add as childs
        listItem.append(controlButton, observeButton)
        this.object.append(listItem)
        // listItem as class
        let listItemObject = new ConnectionListItem(listItem, name)
        this.list[name] = listItemObject

        listItemObject.setButtonsVisibility(has_control, is_client)
    }

    remove(name) {
        this.object.removeChild(this.list[name].object)
        this.list[name] = null    
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
        this.is_observer = false
        this.name = name

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
            type: "set_observer",
            is_observer: removing,
            target: this.name
        })
    }

    controlButtonClicked() {
        this.controlButton.hidden = true
        invoke({
            type: "transfer_control",
            target: this.name
        })
    }

    setButtonsVisibility(hasControl, isClient) {
        this.controlButton.hidden = this.is_observer || (!hasControl && !this.is_observer)
        this.observeButton.hidden = isClient
    }
}