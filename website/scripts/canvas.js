var canvas = document.getElementById("canvas");
var context = canvas.getContext("2d");

var scale = 2;
var C = 16 * scale;
var R = 9 * scale;
var cellSize = screen.height / R;
var cellCounts;
var scales;
var cellDims;


//console.log("Screen: %d x %d, Window: %d x %d", screen.width, screen.height, window.innerWidth, window.innerHeight);
//console.log("(%d, %d)", canvas.width / C, canvas.height / R)

refreshGrid = function() {
    resizeGrid();
    drawGrid();
    fillCells();
}

refreshGrid();

window.onresize = refreshGrid;

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
    context.lineWidth = 5;
    context.strokeStyle = "black";
    
    for (var c = 0; c < cellCounts.x; c++) {
        for (var r = 0; r < cellCounts.y; r++) {
            context.beginPath();
            context.rect(c * cellDims.x, r * cellDims.y, cellDims.x, cellDims.y);
            context.stroke();
        }
    }
}

function fillCells() {
    // cells = [{x (rel. to screen): , y (rel. to screen): , type: , teamColor: }, {}, ...]
    // cells = getCellsFromRust(screen min X coord, screen min Y coord, cellCounts.x, cellCounts.y);
    cells = [
        {x: 4, y: 4, type: "queen", color: "green"},
        {x: 5, y: 4, type: "base", color: "green"},
        {x: 10, y: 5, type: "queen", color: "red"}
    ]

    context.lineWidth = 5;
    context.strokeStyle = "black";
    
    for (var i = 0; i < cells.length; i++) {
        //console.log("Drawing %o", cells[i]);
        context.fillStyle = cells[i].color;
        context.beginPath();
        context.rect(cells[i].x * cellDims.x, cells[i].y * cellDims.y, cellDims.x, cellDims.y);
        context.fill();
        context.stroke();
    }
}

document.onclick = function fillSquare(event) {
    //console.log("Begin fillSquare()");
    context.lineWidth = 5;
    context.strokeStyle = "black";
    context.fillStyle = "green";
    var x = event.clientX;
    var y = event.clientY;
    x = x - (x % (cellDims.x));
    y = y - (y % (cellDims.y));
    //console.log("(%d, %d) --> (%d, %d)", x, y, c, r);
    context.beginPath();
    context.rect(x, y, cellDims.x, cellDims.y);
    context.fill();
    context.stroke();
}

function shiftCanvas(shiftX, shiftY) {
    context.globalCompositeOperation = "copy";
    context.drawImage(canvas, shiftX, shiftY);
    context.globalCompositeOperation = "source-over";
    refreshGrid();
}

document.onkeydown = function shiftView(event) {
    //console.log("%s key pressed", event.code);
    switch (event.code) {
        case "ArrowUp":
            shiftCanvas(0, canvas.height / 2);
            break;
        case "ArrowDown":
            shiftCanvas(0, -canvas.height / 2);
            break;
        case "ArrowLeft":
            shiftCanvas(canvas.width / 2, 0);
            break;
        case "ArrowRight":
            shiftCanvas(-canvas.width / 2, 0);
            break;
    }
}
