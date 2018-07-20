const canvas = document.getElementById("monitorCanvas");
const context = canvas.getContext("2d");

const hitCanvas = document.createElement("canvas");
hitCanvas.width = canvas.width;
hitCanvas.height = canvas.height;
const hitCtx = hitCanvas.getContext("2d");

const unselected = "#333";
const selected = "#8f0003";
const configured = "#0044ad";
const selected_option = "#00dcf3";
const multiplier = 0.122;
const width = 10;

let color_idx = {};
let selected_color_idx = {};

const port_modal = document.getElementById("portInfo");
const background_fade = document.getElementById("background_darken");

const ip_num = document.getElementById("ip_num");
const port_num = document.getElementById("port_num");

// document.addEventListener('contextmenu', function(ev) {
//     external.invoke("debug " + ev.button);
//     ev.preventDefault();
// }, false);

canvas.addEventListener("click", onClick);
canvas.addEventListener("contextmenu", onClick);

background_fade.addEventListener("click", removeFade);
background_fade.addEventListener("contextmenu", removeFade);

let current_color = null;

function removeFade(ev) {
    port_modal.classList.remove("popping");
    background_fade.classList.remove("popping");
    color_idx[current_color].connection_info = valOrPlaceholder(ip_num) + ":" + valOrPlaceholder(port_num);
    selected_color_idx[current_color].connection_info = color_idx[current_color].connection_info;
    idxToLine(selected_color_idx[current_color], configured);
}

function onClick(ev) {

    const mousePos = {
        x: ev.clientX - canvas.offsetLeft - 6,
        y: ev.clientY - canvas.offsetTop - 6
    };
    // external.invoke("debug " + JSON.stringify(mousePos));
    const pixel = hitCtx.getImageData(mousePos.x, mousePos.y, 1, 1).data;
    const color = rgb(pixel[0], pixel[1], pixel[2]);

    let current_idx = color_idx[color];
    if (!current_idx) {
        return;
    }

    if (ev.button === 0) {
        if (selected_color_idx[color]) { // Already selected
            delete selected_color_idx[color];
            idxToLine(current_idx, unselected);
            // external.invoke("debug " + JSON.stringify(color_idx));
        } else { // Not selected yet
            selected_color_idx[color] = current_idx;
            idxToLine(current_idx, selected);
        }
    } else if (ev.button === 2) {
        if (!selected_color_idx[color]) { // Not selected yet
            selected_color_idx[color] = current_idx;
        }
        idxToLine(current_idx, selected_option);
        port_modal.style.left = Math.min(ev.clientX, window.innerWidth - port_modal.offsetWidth) + "px";
        port_modal.style.top = Math.min(ev.clientY, window.innerHeight - port_modal.offsetHeight) + "px";
        ip_num.value = "";
        port_num.value = "";
        port_modal.classList.add("popping");
        background_fade.classList.add("popping");
        current_color = color;
        // external.invoke("debug " + window.innerWidth + " " + port_modal.style.width);
    }
}

function rgb(r, g, b) {
    r = Math.floor(r);
    g = Math.floor(g);
    b = Math.floor(b);
    return ["rgb(", r, ",", g, ",", b, ")"].join("");
}

function showConnectedMsg(e) {
    document.getElementById('is_connected').innerHTML = e;
}

function valOrPlaceholder(item) {
    if (item.value) {
        return item.value;
    } else {
        return item.placeholder;
    }
}

function idxToLine(idx, color) {
    let side = idx.pos_info;
    switch (side.outward_direction) {
        case "Up":
            drawVisibleLine(side.position, side.length, 1, color);
            break;
        case "Down":
            drawVisibleLine(side.position, -side.length, -1, color);
            break;
        case "Left":
            drawVisibleLine(side.position, 1, -side.length, color);
            break;
        case "Right":
            drawVisibleLine(side.position, -1, side.length, color);
    }
}

function drawVisibleLine(position, length_x, length_y, color) {
    context.beginPath();
    context.rect(position.x * multiplier + 0.5, position.y * multiplier + 0.5, Math.abs(length_x) === 1 ? length_x * width : (length_x * multiplier), Math.abs(length_y) === 1 ? length_y * width : (length_y * multiplier));
    context.fillStyle = color;
    context.fill();
}

function drawLineInit(position, length_x, length_y) {
    let color = getRandomColor();
    while (color_idx[color]) {
        color = getRandomColor();
    }

    hitCtx.beginPath();
    hitCtx.rect(position.x * multiplier + 0.5, position.y * multiplier + 0.5, Math.abs(length_x) === 1 ? length_x * width : (length_x * multiplier), Math.abs(length_y) === 1 ? length_y * width : (length_y * multiplier));
    hitCtx.fillStyle = color;
    // external.invoke("debug " + JSON.stringify(color));
    hitCtx.fill();

    drawVisibleLine(position, length_x, length_y, unselected);

    return color;
}

function getRandomColor() {
    let r = Math.round(Math.random() * 255);
    let g = Math.round(Math.random() * 255);
    let b = Math.round(Math.random() * 255);
    return "rgb(" + r + "," + g + "," + b + ")";
}

function connect() {
    external.invoke('connect ' + JSON.stringify(selected_color_idx));
}

function showMonitorList(json_obj) {
    for (let i = 0; i < json_obj.length; i++) {
        let sides = json_obj[i].sides;

        color_idx[drawLineInit(sides.Up.position, sides.Up.length, 1)] = {pos_info: sides.Up};
        color_idx[drawLineInit(sides.Down.position, -sides.Down.length, -1)] = {pos_info: sides.Down};
        color_idx[drawLineInit(sides.Left.position, 1, -sides.Left.length)] = {pos_info: sides.Left};
        color_idx[drawLineInit(sides.Right.position, -1, sides.Right.length)] = {pos_info: sides.Right};
    }
}

external.invoke("loaded");