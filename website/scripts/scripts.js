var WS = new WebSocket("ws://10.8.36.184:2794");
var UID;
WS.onopen = event => {
    console.log("Connected to %s", ws.url)
};
WS.onclose = event => {
    console.log("Disconnected from %s", ws.url)
};
WS.onmessage = event => {
    var payload = JSON.parse(event.data);
    switch (event.type) {
        case "IDENTIFY":
            UID = payload.id;
            console.log("Client UID: %s", UID);
            break;
        case "FRAME":
            fillCells(payload);
            break;
    }
};

var USERNAME;
var launched = false;
function launchGame() {
    USERNAME = document.getElementById("input").value;
    document.getElementById("landing").remove();
    launched = true;
    refreshGrid();
    document.getElementById("username").innerHTML = USERNAME;
}


var canvas = document.getElementById("canvas");
var ctx = canvas.getContext("2d");

var scale = 2;
var C = 16 * scale;
var R = 9 * scale;
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

function getRandomCells(numCells) {
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
            pxX = x * cellDims.x;
            pxY = y * cellDims.y;

            ctx.beginPath();
            ctx.rect(pxX, pxY, cellDims.x, cellDims.y);
            
            cellTeam = cell.team;
            ctx.fillStyle = cellTeam == UID ? "green" : "red";

            ctx.stroke();
            ctx.fillStyle = "white";
            var type = cell.tile;
            if (type != "EMPTY") {
                ctx.fillText(type, pxX + (cellSize / 2), pxY + (cellSize / 2));
            }
        }
    }

    /*var x, y;
    for (var i = 0; i < cells.length; i++) {
        //console.log("Drawing %o", cells[i]);
        x = cells[i].x * cellDims.x;
        y = cells[i].y * cellDims.y;

        ctx.beginPath();
        ctx.rect(x, y, cellDims.x, cellDims.y);
        ctx.fillStyle = cells[i].color;
        ctx.fill();
        ctx.stroke();
        ctx.fillStyle = "white";
        ctx.fillText(cells[i].type, x + (cellSize / 2), y + (cellSize / 2));
    }*/
}

/*document.onclick = function fillSquare(event) {
    //console.log("Begin fillSquare()");
    ctx.lineWidth = cellLineWidth;
    ctx.strokeStyle = "black";
    ctx.fillStyle = "green";
    var x = event.clientX;
    var y = event.clientY;
    x = x - (x % (cellDims.x));
    y = y - (y % (cellDims.y));
    //console.log("(%d, %d) --> (%d, %d)", x, y, c, r);
    ctx.beginPath();
    ctx.rect(x, y, cellDims.x, cellDims.y);
    ctx.fill();
    ctx.stroke();
}*/

function shiftCanvas(shiftX, shiftY, time) {
    shifting = true;
    var fps = 30;
    var frames = fps * time;
    var interval = setInterval(function() {instantShiftCanvas(shiftX / frames, shiftY / frames)}, time * 1000 / frames);
    setTimeout(function() {
        clearInterval(interval);
        shifting = false;
        refreshGrid();
    }, time * 1000);
}

function instantShiftCanvas(shiftX, shiftY) {
    console.log("shift: (%d, %d)", shiftX, shiftY);
    ctx.globalCompositeOperation = "copy";
    ctx.drawImage(ctx.canvas, shiftX, shiftY);
    ctx.globalCompositeOperation = "source-over"
}

document.onkeydown = function shiftView(event) {
    if (shifting) return;
    var time = 0.4;
    switch (event.code) {
        case "ArrowUp":
            shiftCanvas(0, canvas.height / 2, time);
            break;
        case "ArrowDown":
            shiftCanvas(0, -canvas.height / 2, time);
            break;
        case "ArrowLeft":
            shiftCanvas(canvas.width / 2, 0, time);
            break;
        case "ArrowRight":
            shiftCanvas(-canvas.width / 2, 0, time);
            break;
    }
}
