// Builds a wavy surface, shifts a copy of it, and lets congru find the
// shift back through the C++ wrapper.
#include <cmath>
#include <cstdio>
#include <vector>

#include "congru.hpp"

int main() {
    std::vector<double> model;
    std::vector<double> scan;
    for (int row = 0; row < 40; ++row) {
        for (int column = 0; column < 40; ++column) {
            const double along = column / 39.0;
            const double across = row / 39.0;
            const double height = 0.2 * std::sin(3.0 * along) * std::cos(2.0 * across);
            model.insert(model.end(), {along, across, height});
            scan.insert(scan.end(), {along + 0.02, across - 0.01, height + 0.015});
        }
    }
    const congru::target target(model);
    const congru_alignment alignment = target.align(scan);
    std::printf("translation %.4f %.4f %.4f, converged %d after %u iterations\n",
                alignment.transform[3], alignment.transform[7], alignment.transform[11],
                alignment.converged, alignment.iterations);
    return 0;
}
