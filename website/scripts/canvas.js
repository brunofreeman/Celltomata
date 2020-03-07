var canvas = document.getElementById("canvas");
var context = canvas.getContext("2d");

canvas.width  = window.innerWidth;
canvas.height = window.innerHeight;

var scale = 2;
var C = 16 * scale;
var R = 9 * scale;

drawGrid();

function drawGrid() {
    context.lineWidth = 5;
    context.strokeStyle = "black";
    
    for (var c = 0; c < C; c++) {
        for (var r = 0; r < R; r++) {
            context.beginPath();
            context.rect(c * canvas.width / C, r * canvas.height / R, canvas.width / C, canvas.height / R);
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
    drawGrid();
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