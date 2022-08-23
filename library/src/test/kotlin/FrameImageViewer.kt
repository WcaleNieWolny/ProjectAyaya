package me.wcaleniewolny.ayaya

import me.wcaleniewolny.ayaya.library.SplittedFrame
import java.awt.Color
import java.awt.Dimension
import java.awt.Graphics
import javax.swing.JFrame
import javax.swing.JPanel

//Note: This class is not being used at the time, but it is a reference for implementing this in minecraft later
class FrameAwtGui() : JFrame() {

    public constructor(frames: List<SplittedFrame>, width: Int, height: Int) : this() {
        add(FrameImagePanel(frames, width, height))
        title = "test"
        this.defaultCloseOperation = EXIT_ON_CLOSE;
        isResizable = false;
        pack()
        isVisible = true
        setLocationRelativeTo(null)
    }
}

class FrameImagePanel(private val frames: List<SplittedFrame>, private val imgWidth: Int, imgHeight: Int) : JPanel() {

    private fun c(r: Int, g: Int, b: Int): Color {
        return Color(r, g, b)
    }

    val colors = arrayOf(
        c(0, 0, 0),
        c(0, 0, 0),
        c(0, 0, 0),
        c(0, 0, 0),
        c(89, 125, 39),
        c(109, 153, 48),
        c(127, 178, 56),
        c(67, 94, 29),
        c(174, 164, 115),
        c(213, 201, 140),
        c(247, 233, 163),
        c(130, 123, 86),
        c(140, 140, 140),
        c(171, 171, 171),
        c(199, 199, 199),
        c(105, 105, 105),
        c(180, 0, 0),
        c(220, 0, 0),
        c(255, 0, 0),
        c(135, 0, 0),
        c(112, 112, 180),
        c(138, 138, 220),
        c(160, 160, 255),
        c(84, 84, 135),
        c(117, 117, 117),
        c(144, 144, 144),
        c(167, 167, 167),
        c(88, 88, 88),
        c(0, 87, 0),
        c(0, 106, 0),
        c(0, 124, 0),
        c(0, 65, 0),
        c(180, 180, 180),
        c(220, 220, 220),
        c(255, 255, 255),
        c(135, 135, 135),
        c(115, 118, 129),
        c(141, 144, 158),
        c(164, 168, 184),
        c(86, 88, 97),
        c(106, 76, 54),
        c(130, 94, 66),
        c(151, 109, 77),
        c(79, 57, 40),
        c(79, 79, 79),
        c(96, 96, 96),
        c(112, 112, 112),
        c(59, 59, 59),
        c(45, 45, 180),
        c(55, 55, 220),
        c(64, 64, 255),
        c(33, 33, 135),
        c(100, 84, 50),
        c(123, 102, 62),
        c(143, 119, 72),
        c(75, 63, 38),
        c(180, 177, 172),
        c(220, 217, 211),
        c(255, 252, 245),
        c(135, 133, 129),
        c(152, 89, 36),
        c(186, 109, 44),
        c(216, 127, 51),
        c(114, 67, 27),
        c(125, 53, 152),
        c(153, 65, 186),
        c(178, 76, 216),
        c(94, 40, 114),
        c(72, 108, 152),
        c(88, 132, 186),
        c(102, 153, 216),
        c(54, 81, 114),
        c(161, 161, 36),
        c(197, 197, 44),
        c(229, 229, 51),
        c(121, 121, 27),
        c(89, 144, 17),
        c(109, 176, 21),
        c(127, 204, 25),
        c(67, 108, 13),
        c(170, 89, 116),
        c(208, 109, 142),
        c(242, 127, 165),
        c(128, 67, 87),
        c(53, 53, 53),
        c(65, 65, 65),
        c(76, 76, 76),
        c(40, 40, 40),
        c(108, 108, 108),
        c(132, 132, 132),
        c(153, 153, 153),
        c(81, 81, 81),
        c(53, 89, 108),
        c(65, 109, 132),
        c(76, 127, 153),
        c(40, 67, 81),
        c(89, 44, 125),
        c(109, 54, 153),
        c(127, 63, 178),
        c(67, 33, 94),
        c(36, 53, 125),
        c(44, 65, 153),
        c(51, 76, 178),
        c(27, 40, 94),
        c(72, 53, 36),
        c(88, 65, 44),
        c(102, 76, 51),
        c(54, 40, 27),
        c(72, 89, 36),
        c(88, 109, 44),
        c(102, 127, 51),
        c(54, 67, 27),
        c(108, 36, 36),
        c(132, 44, 44),
        c(153, 51, 51),
        c(81, 27, 27),
        c(17, 17, 17),
        c(21, 21, 21),
        c(25, 25, 25),
        c(13, 13, 13),
        c(176, 168, 54),
        c(215, 205, 66),
        c(250, 238, 77),
        c(132, 126, 40),
        c(64, 154, 150),
        c(79, 188, 183),
        c(92, 219, 213),
        c(48, 115, 112),
        c(52, 90, 180),
        c(63, 110, 220),
        c(74, 128, 255),
        c(39, 67, 135),
        c(0, 153, 40),
        c(0, 187, 50),
        c(0, 217, 58),
        c(0, 114, 30),
        c(91, 60, 34),
        c(111, 74, 42),
        c(129, 86, 49),
        c(68, 45, 25),
        c(79, 1, 0),
        c(96, 1, 0),
        c(112, 2, 0),
        c(59, 1, 0),
        c(147, 124, 113),
        c(180, 152, 138),
        c(209, 177, 161),
        c(110, 93, 85),
        c(112, 57, 25),
        c(137, 70, 31),
        c(159, 82, 36),
        c(84, 43, 19),
        c(105, 61, 76),
        c(128, 75, 93),
        c(149, 87, 108),
        c(78, 46, 57),
        c(79, 76, 97),
        c(96, 93, 119),
        c(112, 108, 138),
        c(59, 57, 73),
        c(131, 93, 25),
        c(160, 114, 31),
        c(186, 133, 36),
        c(98, 70, 19),
        c(72, 82, 37),
        c(88, 100, 45),
        c(103, 117, 53),
        c(54, 61, 28),
        c(112, 54, 55),
        c(138, 66, 67),
        c(160, 77, 78),
        c(84, 40, 41),
        c(40, 28, 24),
        c(49, 35, 30),
        c(57, 41, 35),
        c(30, 21, 18),
        c(95, 75, 69),
        c(116, 92, 84),
        c(135, 107, 98),
        c(71, 56, 51),
        c(61, 64, 64),
        c(75, 79, 79),
        c(87, 92, 92),
        c(46, 48, 48),
        c(86, 51, 62),
        c(105, 62, 75),
        c(122, 73, 88),
        c(64, 38, 46),
        c(53, 43, 64),
        c(65, 53, 79),
        c(76, 62, 92),
        c(40, 32, 48),
        c(53, 35, 24),
        c(65, 43, 30),
        c(76, 50, 35),
        c(40, 26, 18),
        c(53, 57, 29),
        c(65, 70, 36),
        c(76, 82, 42),
        c(40, 43, 22),
        c(100, 42, 32),
        c(122, 51, 39),
        c(142, 60, 46),
        c(75, 31, 24),
        c(26, 15, 11),
        c(31, 18, 13),
        c(37, 22, 16),
        c(19, 11, 8),
        c(133, 33, 34),
        c(163, 41, 42),
        c(189, 48, 49),
        c(100, 25, 25),
        c(104, 44, 68),
        c(127, 54, 83),
        c(148, 63, 97),
        c(78, 33, 51),
        c(64, 17, 20),
        c(79, 21, 25),
        c(92, 25, 29),
        c(48, 13, 15),
        c(15, 88, 94),
        c(18, 108, 115),
        c(22, 126, 134),
        c(11, 66, 70),
        c(40, 100, 98),
        c(50, 122, 120),
        c(58, 142, 140),
        c(30, 75, 74),
        c(60, 31, 43),
        c(74, 37, 53),
        c(86, 44, 62),
        c(45, 23, 32),
        c(14, 127, 93),
        c(17, 155, 114),
        c(20, 180, 133),
        c(10, 95, 70),
        c(70, 70, 70),
        c(86, 86, 86),
        c(100, 100, 100),
        c(52, 52, 52),
        c(152, 123, 103),
        c(186, 150, 126),
        c(216, 175, 147),
        c(114, 92, 77),
        c(89, 117, 105),
        c(109, 144, 129),
        c(127, 167, 150),
        c(67, 88, 79)
    )

    val indexMap = mutableMapOf<Int, Color>()

    init { //AWT Stuff

        println("B: ${matchColor(Color(13, 13, 13))}")
        preferredSize = Dimension(128, 128)
        isFocusable = true;
        repaint()
        generateMap()
        background = Color.RED
    }


    private fun distance(c1: Color, c2: Color): Double {
        val ra = (c1.red + c2.red) / 2.0
        val rd = c1.red - c2.red
        val gd = c1.green - c2.green
        val bd = c1.blue - c2.blue
        val weightR = 2 + ra / 256.0
        val weightG = 4.0
        val weightB = 2 + (255 - ra) / 256.0
        return weightR * rd * rd + weightG * gd * gd + weightB * bd * bd
    }

    fun matchColor(color: Color): Byte {
        if (color.alpha < 128) return 0

        var index = 0
        var best = -1.0
        for (i in 4 until colors.size) {
            val distance: Double = distance(color, colors[i])
            if (distance < best || best == -1.0) {
                best = distance
                index = i
            }
        }

        // Minecraft has 143 colors, some of which have negative byte representations
        val finalByte = (if (index < 128) index else -129 + (index - 127)).toByte()
        return finalByte
    }

    private fun generateMap() {
        for (i in 4 until colors.size) {
            val finalIndex = (if (i < 128) i else -129 + (i - 127))
            indexMap[finalIndex] = colors[i]
        }
    }

    override fun paintComponent(g: Graphics) {
        super.paintComponent(g)
        val frame = frames[1];

        for (x in 0 until frame.width) {
            for (y in 0 until frame.height) {
                val byte = frame.data[(y * frame.width) + x]
                val finalColor = indexMap[byte.toInt()]

                g.color = finalColor
                g.drawLine(frame.startX + x, frame.startY + y, frame.startX + x, frame.startY + y)
            }
        }
    }
    //14, 34, 119
    //34 = white

}