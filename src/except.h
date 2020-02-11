#ifndef EXCEPT_H
#define EXCEPT_H

#include <string>
#include <stdexcept>

class bt_error : public std::runtime_error {
    public:
    bt_error(const std::string &arg);
};

#endif
