// congru for C++20: RAII over the UNSTABLE C ABI of congru.h; failures
// throw congru::error carrying the library's message.
#pragma once

#include <array>
#include <cstdint>
#include <span>
#include <stdexcept>
#include <string>

#include "congru.h"

namespace congru {

class error : public std::runtime_error {
  public:
    explicit error(congru_status status)
        : std::runtime_error(std::string(congru_status_message(status)) + ": " +
                             congru_last_error_message()) {}
};

inline void check(congru_status status) {
    if (status != CONGRU_OK) {
        throw error(status);
    }
}

using transform = std::array<double, 16>;

class target {
  public:
    explicit target(std::span<const double> xyz, std::uint32_t normal_neighbors = 10) {
        check(congru_target_new_from_points(xyz.data(), xyz.size() / 3, normal_neighbors,
                                            &handle_));
    }
    target(const target &) = delete;
    target &operator=(const target &) = delete;
    ~target() { congru_target_free(handle_); }

    congru_alignment align(std::span<const double> xyz, const transform *initial = nullptr,
                           const congru_icp_options *options = nullptr) const {
        congru_alignment out{};
        check(congru_icp_align(handle_, xyz.data(), xyz.size() / 3,
                               initial ? initial->data() : nullptr, options, &out));
        return out;
    }

  private:
    congru_target *handle_ = nullptr;
};

} // namespace congru
