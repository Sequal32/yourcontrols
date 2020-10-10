var aircraftList = document.getElementById("aircraft-list")

aircraftList.addAircraft = function(aircraftName) {
    const newButton = document.createElement("button")
    newButton.type = "button"
    newButton.className = "list-group-item list-group-item-action aircraft-list-entry"
    newButton.innerHTML = aircraftName

    newButton.onclick = function() {
        invoke({"type":"load_aircraft", "name": aircraftName})
        newButton.classList.add("active")
    }

    if (aircraftList.activeSelection) {
        aircraftList.activeSelection.classList.remove("active")
    }
    aircraftList.activeSelection = newButton

    aircraftList.appendChild(newButton)
}