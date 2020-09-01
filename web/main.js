var connect_button = document.getElementById('connect-button')
var server_button = document.getElementById('server-button')
var control_button = document.getElementById('control-button')
var alert = document.getElementById("alert")

var server_page_button = document.getElementById("server-page")
var client_page_button = document.getElementById("client-page")

var port_input = document.getElementById("port-input")
var server_input = document.getElementById("server-input")

var trying_connection = false
var is_connected = false
var on_client = true
var has_control = false

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
    
    server_input.disabled = true
    port_input.disabled = true
    is_connected = true
}

function PageChange(isClient) {
    on_client = isClient
    connect_button.hidden = !isClient
    server_button.hidden = isClient
    server_input.hidden = !isClient
    ResetForm()
}


function PagesVisible(visible) {
    document.getElementById("nav").hidden = !visible
}

function ValidatePort(str) {
    return str.match(/\d+/gi)
}

function ValidateIp(str) {
    return str.match(/^(?:[0-9]{1,3}\.){3}[0-9]{1,3}$/gi)
}

// Handle server messages
function MessageReceived(data) {
    switch (data["type"]) {
        case "attempt":
            alert.updatetext("warning", "Attempting connection...")
            break;
        case "connected":
            OnConnected()
            alert.updatetext("success", "Connected to server.")
            connect_button.updatetext("danger", "Disconnect")
            break;
        case "server_failed":
        case "disconnected":
            alert.updatetext("danger", "Not connected.")
            
            trying_connection = false
            is_connected = false
            control_button.hidden = true

            PagesVisible(true)
            ResetForm()
            break;
        case "server":
            OnConnected()
            alert.updatetext("success", `Server started! ${data["data"]} clients connected.`)
            break;
        case "error":
            alert.updatetext("danger", data["data"])
            break;
        case "controlavail":
            control_button.hidden = false
            control_button.innerHTML = "Take Control"
            break;
        case "control":
            has_control = true
            control_button.innerHTML = "Relieve Control"
            control_button.hidden = false
            break;
        case "lostcontrol":
            has_control = false
            control_button.hidden = true
            break;
    }
}

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
    alert.innerHTML = text
}

server_page_button.onclick = function() {
    server_page_button.classList.add("active")
    client_page_button.classList.remove("active")
    PageChange(false)
}

client_page_button.onclick = function() {
    client_page_button.classList.add("active")
    server_page_button.classList.remove("active")
    PageChange(true)
}

control_button.onclick = function() {
    if (has_control) {
        invoke({"type":"relieve"})
    } else {
        invoke({"type":"take"})
    }
}

document.getElementById("main-form").onsubmit = function(e) {
    e.preventDefault()

    if (trying_connection) {return}
    if (is_connected) {invoke({type: "disconnect"}); return}

    var validip = ValidateIp(server_input.value)
    var validport = ValidatePort(port_input.value)

    Validate(port_input, validport)

    if (on_client) {
        Validate(server_input, validip)
        if (!validip || !validport) {return}
        invoke({type: "connect", ip: server_input.value, port: parseInt(port_input.value)})
    } else {
        if (!validport) {return}
        invoke({type: "server", port: parseInt(port_input.value)})
    }
}