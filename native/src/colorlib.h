//
// Created by wolny on 7/12/22.
//

#ifndef WOLNYJNI_COLORLIB_H
#define WOLNYJNI_COLORLIB_H

struct RgbColor{
    unsigned char red;
    unsigned char green;
    unsigned char blue;
};

const struct RgbColor col_getColor(unsigned char red, unsigned char green, unsigned char blue);

static double col_distance(struct RgbColor c1, struct RgbColor c2);

signed char col_get_mc_index(struct RgbColor color);

short col_size();

#endif //WOLNYJNI_COLORLIB_H
