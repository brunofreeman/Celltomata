var canvas = document.getElementById("canvas");
var ctx = canvas.getContext("2d");

var scale = 2;
var C = 16 * scale;
var R = 9 * scale;
var cellSize = screen.height / R;
var cellLineWidth = cellSize / 10;
console.log("Cell size: %d", cellSize);
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

function fillCells() {
    // cells = [{x (rel. to screen): , y (rel. to screen): , type: , teamColor: }, {}, ...]
    // cells = getCellsFromRust(screen min X coord, screen min Y coord, cellCounts.x, cellCounts.y);
    cells = [
        {x: 4, y: 4, type: "queen", color: "green"},
        {x: 5, y: 4, type: "base", color: "green"},
        {x: 10, y: 5, type: "queen", color: "red"}
    ]

    ctx.lineWidth = cellLineWidth;
    ctx.strokeStyle = "black";
    ctx.font = `${cellSize / 3.5}px Arial`;
    ctx.textAlign = "center"; 
    ctx.textBaseline = "middle";//**

    var x, y;
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
    }
}

document.onclick = function fillSquare(event) {
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
}

function shiftCanvas(shiftX, shiftY) {
    ctx.globalCompositeOperation = "copy";
    ctx.drawImage(canvas, shiftX, shiftY);
    ctx.globalCompositeOperation = "source-over";
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
