var WS;
var UID;
var USERNAME;
var USERNAME_REGEX = "(?![_0-9a-zA-Z]+).";
var IP = "127.0.0.1:2794";
var PROTOCOL = "game-of-strife";
var launched = false;
var connected = false;

var canvas = document.getElementById("canvas");
var ctx = canvas.getContext("2d");

var leaderboard = [
    {username : "An" , numCells : 100},
    {username : "Bruno" , numCells : 80},
    {username : "Emily" , numCells : 60},
    {username : "Kaden" , numCells : 40},
    {username : "Tyler" , numCells : 20},
];

var SCALE = 2;
var C = 16 * SCALE;
var R = 9 * SCALE;
var cellSize = screen.height / R;
var cellLineWidth = cellSize / 10;
var cellCounts;
var scales;
var cellDims;
var origin;
var shifting = false;

var CELL_TYPES = [
    "BASE",
    "SPAWNER",
    "FEEDER",
    "BOLSTER",
    "GUARD",
    "ATTACK"
]
var cellTypeSelected = CELL_TYPES[0];
var energy = 2020;

var COSTS = [
    100, //b
    750, //S
    325, //F
    500, //B
    650, //G
    725 //A
];

var submitted = false;

/*function resizeLanding() {
    textFit("play-button-text");
}

function textFit(id) { // Not working
    /var elem = document.getElementById(id);
    var div = elem.children[0];
    console.log("Elem: %o", elem);
    console.log("Div: %o", div);

    console.log("Font size pre: %s", div.style.fontSize);
    while(div.height > elem.height) {
        div.style.fontSize = parseInt((div.style.fontsize) - 1) + "px";
    }

    while( elem.height > div.height) {
        div.style.fontSize = parseInt((div.style.fontsize) + 1) + "px";
    
    console.log("Font size post: %s", div.style.fontSize);
}*/

function launchGame() {
    USERNAME = document.getElementById("username-input").value;
    var matches = USERNAME.match(USERNAME_REGEX);
    if (!USERNAME || USERNAME.length < 3 || USERNAME.length > 15 || (matches && matches.length > 0)) {
        if (USERNAME) {
            document.getElementById("invalid").innerHTML =
                "Invalid username. Must be 3 to 15 characters long and consist of only letters, numbers, and underscores."
        }
        return;
    }

    document.getElementById("landing").remove();
    document.getElementById("username").innerHTML = USERNAME;
    document.getElementById("energy").innerHTML =  "⚡ " + energy;
    loadSelectingInfo();
    launched = true;

    WS = new WebSocket("ws://" + IP, PROTOCOL);
    WS.onopen = event => {
        console.log("Connected to %s", WS.url);
        connected = true;
        WS.send(JSON.stringify({
            type : "NEW_PLAYER",
            username : USERNAME
        }));
    };
    WS.onclose = event => {
        console.log("Disconnected from %s", WS.url);
    };
    WS.onmessage = event => {
        var payload = JSON.parse(event.data);
        //console.log("Got payload", payload);
        switch (payload.type) {
            case "IDENTIFY":
                UID = payload.id;
                //resizeGrid();
                origin = payload.origin;
                resizeGrid();
                shiftView(Math.ceil(-cellCounts.x / 2), Math.ceil(-cellCounts.y / 2), 0);
                //console.log("Client UID: %s", UID);
                break;
            case "FRAME":
                fillCells(payload);
                break;
            case "GENERATION_PING":
                refreshGrid();
                break;
            case "LEADERBORD_UPDATE":
                // TODO: update leaderboard
                refreshLeaderboard(null);
                break;
            case "ENERGY_UPDATE":
                energy = payload.erg;
                document.getElementById("energy").innerHTML = "⚡ " + energy;
                break;
            case "NOTICE":
                break;
        }
    };
    refreshLeaderboard(null); // remove when An implements the update
}

document.onclick = function fillSquare(event) {
    if (!connected || submitted) return;
    var x = event.clientX;
    var y = event.clientY;
    x = Math.floor(x / cellDims.x);
    y = Math.floor(y / cellDims.y);
    cell = {
        tile: cellTypeSelected,
        team: UID,
        pos: {x: x + origin.x, y: y + origin.y}
    }
    if (energy - COSTS[CELL_TYPES.indexOf(cellTypeSelected)] >= 0) {
        // fillCell(cell, x, y);
        submitCell(cell);
    }
}

function submitCell(cell) {
    WS.send(JSON.stringify({
        type : "PUT",
        tile : cell.tile,
        position : {
            x: cell.pos.x,
            y: cell.pos.y
        }
    }));
}

function fillCells(payload) {
    ctx.lineWidth = cellLineWidth;
    ctx.strokeStyle = "black";
    ctx.font = `${cellSize / 1.5}px Arial`;
    ctx.textAlign = "center"; 
    ctx.textBaseline = "middle";

    for (var y = 0; y < payload.y_size; y++) {
        for (var x = 0; x < payload.x_size; x++) {
            fillCell(payload.window[y][x], x, y);       
        }
    }
}

function fillCell(cell, x, y) {
    if (cell.tile != "EMPTY") {
        pxX = x * cellDims.x;
        pxY = y * cellDims.y;
        ctx.beginPath();
        ctx.rect(pxX, pxY, cellDims.x, cellDims.y);
        ctx.stroke();
        ctx.fillStyle = cell.team == UID ? "green" : "red";
        ctx.fill();
        ctx.fillStyle = "white";
        ctx.fillText(cell.tile[0], pxX + (cellSize / 2), pxY + (cellSize / 2));
    }
}

function loadSelectingInfo() {
    siDOM = document.getElementById("selecting-info");
    siDOM.innerHTML = "";
    for (var i = 0; i < CELL_TYPES.length; i++) {
        var siEntry = document.createElement("LI");
        siEntry.appendChild(document.createTextNode(`${cellTypeSelected == CELL_TYPES[i] ? "> " : ""}${CELL_TYPES[i]} (Cost: ${COSTS[i]})`))
        siDOM.appendChild(siEntry);
    }
}

function refreshLeaderboard(payload) {
    lbDOM = document.getElementById("leaderboard");
    lbDOM.innerHTML = "";
    for (var i = 0; i < leaderboard.length; i++) {
        var lbEntry = document.createElement("LI");
        lbEntry.appendChild(document.createTextNode(`${leaderboard[i].username}: ${leaderboard[i].numCells}`))
        lbDOM.appendChild(lbEntry);
    }
}

function refreshGrid() {
    resizeGrid();
    drawGrid();
    requestCells();
}

window.onresize = function() {
    if (!launched) resizeLanding();
    if (connected) refreshGrid();
}

function resizeGrid() {
    canvas.width  = window.innerWidth;
    canvas.height = window.innerHeight;
    cellCounts = {x: Math.floor(canvas.width / cellSize), y: Math.floor(canvas.height / cellSize)};
    scales = {x: canvas.width / cellSize / cellCounts.x, y: canvas.height / cellSize / cellCounts.y};
    updateCellDims();
}

function updateCellDims() {
    cellDims = {x: cellSize * scales.x, y: cellSize * scales.y};
}

function drawGrid() {
    ctx.lineWidth = cellLineWidth;
    ctx.strokeStyle = "black";
    
    for (var c = 0; c < cellCounts.x; c++) {
        for (var r = 0; r < cellCounts.y; r++) {
            ctx.beginPath();
            ctx.rect(c * cellDims.x, r * cellDims.y, cellDims.x, cellDims.y);
            ctx.stroke();
        }
    }
}

function requestCells() {
    WS.send(JSON.stringify({
        type : "REQUEST_FRAME",
        x_origin : origin.x,
        y_origin : origin.y,
        x_size : cellCounts.x,
        y_size : cellCounts.y
    }));
}

function shiftView(shiftX, shiftY, time) {
    //console.log("Shifting (%d, %d)", shiftX, shiftY);
    if (origin.x + shiftX < 0) shiftX = -origin.x;
    if (origin.x + shiftX > 99) shiftX = 99 - origin.x;
    if (origin.y + shiftY < 0) shiftY = -origin.y;
    if (origin.y + shiftY > 99) shiftY = 99 - origin.y;
    //console.log("Shifting (%d, %d), origin (%d, %d)", shiftX, shiftY, origin.x, origin.y);
    var fps = 30;
    var frames = fps * time;
    ctx.globalCompositeOperation = "copy";
    if (time != 0) {
        shifting = true;
        var interval = setInterval(function() {
            ctx.drawImage(ctx.canvas, cellDims.x * -shiftX / frames, cellDims.y * -shiftY / frames)
        }, time * 1000 / frames);
        setTimeout(function() {
            clearInterval(interval);
            ctx.globalCompositeOperation = "source-over";
            shifting = false;
            origin.x += shiftX;
            origin.y += shiftY;
            refreshGrid();
        }, time * 1000);
    } else {
        ctx.drawImage(ctx.canvas, cellDims.x * -shiftX, cellDims.y * -shiftY);
        ctx.globalCompositeOperation = "source-over";
        origin.x += shiftX;
        origin.y += shiftY;
        refreshGrid();
    }
    
}

document.onkeydown = function keyResponse(event) {
    //console.log(event.code);
    if (shifting) return;
    var TIME = 0.4;
    switch (event.code) {
        case "ArrowUp":
            if (connected) shiftView(0, Math.ceil(-cellCounts.y / 2), TIME);
            break;
        case "ArrowDown":
            if (connected) shiftView(0, Math.floor(cellCounts.y / 2), TIME);
            break;
        case "ArrowLeft":
            if (connected) shiftView(Math.ceil(-cellCounts.x / 2), 0, TIME);
            break;
        case "ArrowRight":
            if (connected) shiftView(Math.floor(cellCounts.x / 2), 0, TIME);
            break;
        case "Enter":
            if (!launched) launchGame();
            else if (!submitted) {
                submitCells();
                submitted = true;
            }
            break;
        /*case "Space": // for testing/debugging
            if (connected) refreshGrid();
            break;*/
        case "Digit1":
            cellTypeSelected = CELL_TYPES[0];
            loadSelectingInfo();
            break;
        case "Digit2":
            cellTypeSelected = CELL_TYPES[1];
            loadSelectingInfo();
            break;     
        case "Digit3":
            cellTypeSelected = CELL_TYPES[2];
            loadSelectingInfo();
            break;
        case "Digit4":
            cellTypeSelected = CELL_TYPES[3];
            loadSelectingInfo();
            break;
        case "Digit5":
            cellTypeSelected = CELL_TYPES[4];
            loadSelectingInfo();
            break;
        case "Digit6":
            cellTypeSelected = CELL_TYPES[5];
            loadSelectingInfo();
            break;
    }
}

/*function getRandomCells(numCells) {
    var types = ["Q", "b", "F", "S"];
    var colors = ["red", "green", "blue"];
    var cells = [];
    var x, y, type, color;
    for (var i = 0; i < numCells; i++) {
        x = Math.floor(Math.random() * cellCounts.x);
        y = Math.floor(Math.random() * cellCounts.y);
        type = types[Math.floor(Math.random() * types.length)];
        color = colors[Math.floor(Math.random() * colors.length)];
        cells.push({x: x, y: y, type: type, color: color})
    }
    return cells;
}*/