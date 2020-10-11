var connect_button = document.getElementById('connect-button')
var server_button = document.getElementById('server-button')
var alert = document.getElementById("alert")
var overloaded_alert = document.getElementById("overloaded-alert")

var nav_bar = document.getElementById("nav")
var server_client_page = document.getElementById("server-client-page");
var server_page_button = document.getElementById("server-page")
var client_page_button = document.getElementById("client-page")
var connection_list_button = document.getElementById("connection-page")
var aircraft_list_button = document.getElementById("aircraft-page")

var port_input = document.getElementById("port-input")
var server_input = document.getElementById("server-input")
var name_input = document.getElementById("name-input")
var name_div = document.getElementById("name-div")
var port_div = document.getElementById("port-div")
var server_div = document.getElementById("server-div")

var rectangle_status = document.getElementById("rectangle-status")
// Radios
var radios = document.getElementById("radios")
var ip4radio = document.getElementById("ip4")
var ip6radio = document.getElementById("ip6")

var trying_connection = false
var is_connected = false
var is_client = false
var on_client = true
var has_control = false

// Connection List
var connectionList = new ConnectionList(document.getElementById("connection-list"))
// General functions
function invoke(data) {
    window.external.invoke(JSON.stringify(data))
}

function ResetForm() {
    server_input.classList.remove(["is-valid", "is-invalid"])
    port_input.classList.remove(["is-valid", "is-invalid"])
    connect_button.updatetext("success", "Connect")
    server_button.updatetext("primary", "Start Server")
}

function Validate(e, isValid) {
    e.classList.add(isValid ? "is-valid" : "is-invalid")
    e.classList.remove(isValid ? "is-invalid" : "is-valid")
}

function OnConnected() {
    PagesVisible(false)
    connect_button.updatetext("danger", "Disconnect")
    server_button.updatetext("danger", "Stop Server")
    
    trying_connection = false
    server_input.disabled = true
    port_input.disabled = true
    name_input.disabled = true
    ip4radio.disabled = true
    ip6radio.disabled = true
    is_connected = true
}

function OnDisconnect(text) {
    alert.updatetext("danger", text)
    is_connected = false
    trying_connection = false
    server_input.disabled = false
    port_input.disabled = false
    name_input.disabled = false

    ip4radio.disabled = false
    ip6radio.disabled = false

    connectionList.clear()

    PagesVisible(true)
    ResetForm()
}

function ServerClientPageChange(isClient) {
    server_client_page.hidden = false
    on_client = isClient
    connect_button.hidden = !isClient
    server_button.hidden = isClient
    radios.hidden = isClient
    server_div.hidden = !isClient
    port_div.hidden = false
    name_div.hidden = false
}

function PageChange(pageName) {
    connectionList.hide()
    aircraftList.hidden = true

    client_page_button.classList.remove("active")
    server_page_button.classList.remove("active")
    connection_list_button.classList.remove("active")
    aircraft_list_button.classList.remove("active")

    switch (pageName) {
        case "server":
            ServerClientPageChange(false)
            server_page_button.classList.add("active")
            break;
        case "client":
            ServerClientPageChange(true)
            client_page_button.classList.add("active")
            break;
        case "connections":
            server_client_page.hidden = true
            connectionList.show()
            connection_list_button.classList.add("active")
            break
        case "aircraft":
            server_client_page.hidden = true
            aircraftList.hidden = false
            aircraft_list_button.classList.add("active")
            break
    }

    if (!is_connected) {
        ResetForm()
    }
}


function PagesVisible(visible) {
    if (visible) {
        server_page_button.hidden = false
        client_page_button.hidden = false
        aircraft_list_button.hidden = false
    } else {
        if (on_client) {
            server_page_button.hidden = true
        } else {
            client_page_button.hidden = true
        }
        aircraft_list_button.hidden = true
    }
}

function ValidatePort(str) {
    return str.match(/\d+/gi)
}

function ValidateIp(str) {
    return str.match(/^(([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])$/gi) || str.match(/(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))/gi)
}

function ValidateHostname(str) {
    return str.match(/^(([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9\-]*[a-zA-Z0-9])\.)*([A-Za-z0-9]|[A-Za-z0-9][A-Za-z0-9\-]*[A-Za-z0-9])$/gi)
}

// Handle server messages
function MessageReceived(data) {
    switch (data["type"]) {
        case "attempt":
            alert.updatetext("warning", "Attempting connection...")
            break;
        case "connected":
            OnConnected()
            is_client = true
            alert.updatetext("success", "Connected to server.")
            connect_button.updatetext("danger", "Disconnect")
            break;
        case "server_fail":
            OnDisconnect("Server failed to start. Reason: " + data["data"])
            break;
        case "client_fail":
            OnDisconnect("Client disconnected. Reason: " + data["data"])
            break;
        case "disconnected":
            OnDisconnect("Not Connected.")
            break;
        case "server":
            OnConnected()
            is_client = false
            alert.updatetext("success", "Server started! " + data["data"] + " clients connected.")
            break;
        case "error":
            alert.updatetext("danger", data["data"])
            break;
        case "control":
            has_control = true
            connectionList.update()
            connectionList.hideStatusText()
            rectangle_status.style.backgroundColor = "cyan"
            break;
        case "lostcontrol":
            has_control = false
            connectionList.update()
            rectangle_status.style.backgroundColor = "red"
            break;
        case "set_ip":
            server_input.value = data["data"]
            break;
        case "set_port":
            port_input.value = data["data"]
            break;
        case "overloaded":
            overloaded_alert.hidden = false
            break;
        case "stable":
            overloaded_alert.hidden = true
            break;
        case "newconnection":
            connectionList.add(data["data"])
            break;
        case "lostconnection":
            connectionList.remove(data["data"])
            break;
        // Observing
        case "observing":
            rectangle_status.style.backgroundColor = "grey"
            break;
        case "stop_observing":
            rectangle_status.style.backgroundColor = "red"
            break;
        // Other client
        case "set_observing":
            connectionList.setObserver(data["data"], true)
            break;
        case "set_not_observing":
            connectionList.setObserver(data["data"], false)
            break;
        // Other client
        case "set_incontrol":
            connectionList.setInControl(data["data"])
            break;
        // Add possible aircraft
        case "add_aircraft":
            aircraftList.addAircraft(data["data"])
            break;
        case "select_active_config":
            aircraftList.searchSelectActive(data["data"])
            break;
    }
}

invoke({"type":"startup"})

// Buttons functions

connect_button.updatetext = function(typeString, text) {
    connect_button.className = connect_button.className.replace(/btn-\w+/gi, "btn-" + typeString)
    connect_button.innerHTML = text
}

server_button.updatetext = function(typeString, text) {
    server_button.className = server_button.className.replace(/btn-\w+/gi, "btn-" + typeString)
    server_button.innerHTML = text
}

alert.updatetext = function(typeString, text) {
    alert.className = alert.className.replace(/alert-\w+/gi, "alert-" + typeString)
    // Only change text, do not get rid of rectangle
    alert.childNodes[0].nodeValue = text
}

server_page_button.onclick = function() {
    PageChange("server")
}

client_page_button.onclick = function() {
    PageChange("client")
}

connection_list_button.onclick = function() {
    PageChange("connections")
}

aircraft_list_button.onclick = function() {
    PageChange("aircraft")
}

document.getElementById("main-form").onsubmit = function(e) {
    e.preventDefault()

    if (trying_connection) {return}
    if (is_connected) {invoke({type: "disconnect"}); return}

    var validip = ValidateIp(server_input.value)
    var validhostname = ValidateHostname(server_input.value)
    var validport = ValidatePort(port_input.value)
    let validname = name_input.value.trim() != ""

    Validate(port_input, validport)
    Validate(name_input, validname)

    if (on_client) {
        let data = {type: "connect", port: parseInt(port_input.value)}

        Validate(server_input, validip || validhostname || validip6)

        if (!validname || !validport) {return}
        // Match hostname or ip
        if (validhostname) {
            data["hostname"] = server_input.value
        } else if (validip) {
            data["ip"] = server_input.value
        }
        else {
            return
        }
        data["username"] = name_input.value
        trying_connection = true
        invoke(data);
    } else {
        if (!validport || !validname) {return}
        trying_connection = true
        invoke({type: "server", port: parseInt(port_input.value), is_v6: ip6radio.checked, username: name_input.value})
    }
}