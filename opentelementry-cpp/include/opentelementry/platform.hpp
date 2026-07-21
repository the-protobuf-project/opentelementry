#pragma once

#if defined(OPENTELEMENTRY_FREERTOS)
    #define OPENTELEMENTRY_PLATFORM_FREERTOS 1
    #define OPENTELEMENTRY_PLATFORM_POSIX 0
#elif defined(__unix__) || defined(__APPLE__) || defined(__linux__)
    #define OPENTELEMENTRY_PLATFORM_FREERTOS 0
    #define OPENTELEMENTRY_PLATFORM_POSIX 1
#elif defined(_WIN32)
    #define OPENTELEMENTRY_PLATFORM_FREERTOS 0
    #define OPENTELEMENTRY_PLATFORM_POSIX 0
    #define OPENTELEMENTRY_PLATFORM_WINDOWS 1
#else
    #define OPENTELEMENTRY_PLATFORM_FREERTOS 0
    #define OPENTELEMENTRY_PLATFORM_POSIX 0
#endif

#if OPENTELEMENTRY_PLATFORM_FREERTOS
    #include <FreeRTOS.h>
    #include <semphr.h>
    #include <task.h>

    namespace opentelementry::platform {
        using Mutex = SemaphoreHandle_t;

        inline Mutex create_mutex() {
            return xSemaphoreCreateMutex();
        }

        inline void lock_mutex(Mutex& m) {
            xSemaphoreTake(m, portMAX_DELAY);
        }

        inline void unlock_mutex(Mutex& m) {
            xSemaphoreGive(m);
        }

        inline void destroy_mutex(Mutex& m) {
            vSemaphoreDelete(m);
        }

        inline uint64_t get_timestamp_ns() {
            return static_cast<uint64_t>(xTaskGetTickCount()) *
                   (1000000000ULL / configTICK_RATE_HZ);
        }
    }
#else
    #include <mutex>
    #include <chrono>

    namespace opentelementry::platform {
        using Mutex = std::mutex*;

        inline Mutex create_mutex() {
            return new std::mutex();
        }

        inline void lock_mutex(Mutex& m) {
            if (m) m->lock();
        }

        inline void unlock_mutex(Mutex& m) {
            if (m) m->unlock();
        }

        inline void destroy_mutex(Mutex& m) {
            delete m;
            m = nullptr;
        }

        inline uint64_t get_timestamp_ns() {
            auto now = std::chrono::system_clock::now();
            auto duration = now.time_since_epoch();
            return std::chrono::duration_cast<std::chrono::nanoseconds>(duration).count();
        }
    }
#endif

namespace opentelementry::platform {

class ScopedLock {
public:
    explicit ScopedLock(Mutex& m) : mutex_(m) {
        lock_mutex(mutex_);
    }

    ~ScopedLock() {
        unlock_mutex(mutex_);
    }

    ScopedLock(const ScopedLock&) = delete;
    ScopedLock& operator=(const ScopedLock&) = delete;

private:
    Mutex& mutex_;
};

}  // namespace opentelementry::platform
