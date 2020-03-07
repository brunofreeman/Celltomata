var canvas = document.getElementById("canvas");
var context = canvas.getContext("2d");

var scale = 2;
var C = 16 * scale;
var R = 9 * scale;
var cellSize = screen.height / R;
var numXCells;
var numYCells;
var xScale;
var yScale;

resizeGrid();
//console.log("Screen: %d x %d, Window: %d x %d", screen.width, screen.height, window.innerWidth, window.innerHeight);

drawGrid(xScale, yScale);

//console.log("(%d, %d)", canvas.width / C, canvas.height / R)

function drawGrid(cellXScale, cellYScale) {
    context.lineWidth = 5;
    context.strokeStyle = "black";

    cellX = cellSize * cellXScale;
    cellY = cellSize * cellYScale;
    
    for (var c = 0; c * cellX < canvas.width; c++) {
        for (var r = 0; r * cellY < canvas.height; r++) {
            context.beginPath();
            context.rect(c * cellX, r * cellY, cellX, cellY);
            context.stroke();
        }
    }
}

context.fillStyle = "green";

function fillSquare(event) {
    var x = event.clientX;
    var y = event.clientY;
    c = x - (x % (canvas.width / C));
    r = y - (y % (canvas.height / R));
    //console.log("(%d, %d) --> (%d, %d)", x, y, c, r);
    context.beginPath();
    context.rect(c, r, canvas.width / C, canvas.height / R);
    context.fill();
    context.stroke();
    
}

function shiftCanvas(shiftX, shiftY) {
    context.globalCompositeOperation = "copy";
    context.drawImage(canvas, shiftX, shiftY);
    context.globalCompositeOperation = "source-over";
    drawGrid(1);
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

window.onresize = function refreshGrid(event) {
    resizeGrid();
    drawGrid(xScale, yScale);
}

function resizeGrid(event) {
    canvas.width  = window.innerWidth;
    canvas.height = window.innerHeight;
    numXCells = Math.floor(canvas.width / cellSize);
    numYCells = Math.floor(canvas.height / cellSize);
    xScale = canvas.width / cellSize / numXCells;
    yScale = canvas.height / cellSize / numYCells;
}
