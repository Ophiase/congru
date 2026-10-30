/* congru: rigid registration of point clouds. UNSTABLE C ABI: names,
 * structs and values may change in any release until 1.0. Points are
 * n x 3 doubles, row after row; transforms are 4 x 4 doubles, row-major,
 * taking the source onto the target. */
#ifndef CONGRU_H
#define CONGRU_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum congru_status {
    CONGRU_OK = 0,
    CONGRU_NULL_POINTER = 1,
    CONGRU_INVALID_INPUT = 2,
    CONGRU_INVALID_PARAMETER = 3,
    CONGRU_TOO_FEW_CORRESPONDENCES = 4,
    CONGRU_DEGENERATE = 5,
    CONGRU_MISSING_NORMALS = 6,
    CONGRU_PANIC = 7
} congru_status;

enum { CONGRU_METRIC_POINT_TO_POINT = 0, CONGRU_METRIC_POINT_TO_PLANE = 1 };
enum {
    CONGRU_KERNEL_QUADRATIC = 0,
    CONGRU_KERNEL_HUBER = 1,
    CONGRU_KERNEL_CAUCHY = 2,
    CONGRU_KERNEL_GEMAN_MCCLURE = 3,
    CONGRU_KERNEL_TUKEY = 4
};

typedef struct congru_target congru_target;

typedef struct congru_icp_options {
    int metric;
    int kernel;
    double kernel_scale;                /* unit of the points */
    double max_correspondence_distance; /* 0 keeps every pair */
    double overlap;                     /* in (0, 1] */
    uint32_t max_iterations;
    double rotation_tolerance;          /* radians */
    double translation_tolerance;
} congru_icp_options;

typedef struct congru_alignment {
    double transform[16];
    double rmse;
    double inlier_fraction;
    uint32_t iterations;
    bool converged;
    double conditioning; /* NaN without target normals */
    uint32_t weak_motions;
} congru_alignment;

typedef struct congru_fit {
    double transform[16];
    double rmse;
} congru_fit;

congru_icp_options congru_icp_options_default(void);

/* normal_neighbors = 0 builds no normals: point-to-point only. */
congru_status congru_target_new_from_points(const double *xyz, size_t count,
                                            uint32_t normal_neighbors,
                                            congru_target **out);
void congru_target_free(congru_target *target);

/* initial and options may be NULL: identity, and the defaults. */
congru_status congru_icp_align(const congru_target *target, const double *xyz,
                               size_t count, const double *initial,
                               const congru_icp_options *options,
                               congru_alignment *out);

/* weights may be NULL: uniform. */
congru_status congru_fit_rigid(const double *source, const double *target,
                               size_t count, const double *weights,
                               congru_fit *out);

const char *congru_status_message(int status);
/* The detail of this thread's last failure. */
const char *congru_last_error_message(void);

#ifdef __cplusplus
}
#endif

#endif
