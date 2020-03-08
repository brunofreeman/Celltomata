var WS;
var UID;
var USERNAME;
var launched = false;
var IP = "127.0.0.1:2794";
var PROTOCOL = "game-of-strife";

function launchGame() {
    USERNAME = document.getElementById("input").value;
    if (USERNAME === "") return;
    document.getElementById("landing").remove();
    launched = true;
    document.getElementById("username").innerHTML = USERNAME;

    WS = new WebSocket("ws://" + IP, PROTOCOL);
    WS.onopen = event => {
        console.log("Connected to %s", WS.url);
        refreshGrid();
    };
    WS.onclose = event => {
        console.log("Disconnected from %s", WS.url);
    };
    WS.onmessage = event => {
        var payload = JSON.parse(event.data);
        console.log("Got payload", payload);
        switch (payload.type) {
            case "IDENTIFY":
                UID = payload.id;
                console.log("Client UID: %s", UID);
                break;
            case "FRAME":
                fillCells(payload);
                break;
        }
    };
}


var canvas = document.getElementById("canvas");
var ctx = canvas.getContext("2d");

var SCALE = 2;
var C = 16 * SCALE;
var R = 9 * SCALE;
var cellSize = screen.height / R;
var cellLineWidth = cellSize / 10;
var cellCounts;
var scales;
var cellDims;
var shifting = false;

refreshGrid = function() {
    resizeGrid();
    drawGrid();
    requestCells();
}

window.onresize = function() {
    if (launched) refreshGrid();
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
        x_origin : 0,
        y_origin : 0,
        x_size : cellCounts.x,
        y_size : cellCounts.y
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
            cell = payload.window[y][x];
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
    }
}

document.onclick = function fillSquare(event) {
    /*ctx.lineWidth = cellLineWidth;
    ctx.strokeStyle = "black";
    ctx.fillStyle = "green";
    var x = event.clientX;
    var y = event.clientY;
    x = x - (x % (cellDims.x));
    y = y - (y % (cellDims.y));
    ctx.beginPath();
    ctx.rect(x, y, cellDims.x, cellDims.y);
    ctx.fill();
    ctx.stroke();*/
}

function shiftCanvas(shiftX, shiftY, time) {
    shifting = true;
    var fps = 30;
    var frames = fps * time;
    ctx.globalCompositeOperation = "copy";
    var interval = setInterval(function() {
        ctx.drawImage(ctx.canvas, shiftX / frames, shiftY / frames)
    }, time * 1000 / frames);
    setTimeout(function() {
        clearInterval(interval);
        ctx.globalCompositeOperation = "source-over";
        shifting = false;
        refreshGrid();
    }, time * 1000);
}

document.onkeydown = function shiftView(event) {
    if (shifting) return;
    var TIME = 0.4;
    switch (event.code) {
        case "ArrowUp":
            if (launched) shiftCanvas(0, canvas.height / 2, TIME);
            break;
        case "ArrowDown":
            if (launched) shiftCanvas(0, -canvas.height / 2, TIME);
            break;
        case "ArrowLeft":
            if (launched) shiftCanvas(canvas.width / 2, 0, TIME);
            break;
        case "ArrowRight":
            if (launched) shiftCanvas(-canvas.width / 2, 0, TIME);
            break;
        case "Enter":
            if (!launched) launchGame();
            break;
        case "Space": // for testing/debugging
            if (launched) refreshGrid();
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